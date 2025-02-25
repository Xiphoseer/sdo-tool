//! High-Level API

use std::{
    borrow::Cow,
    collections::BTreeMap,
    io::{self, Write},
    marker::PhantomData,
    str::FromStr,
};

use chrono::Local;
use uuid::Uuid;

use crate::{
    common::{
        self, Dict, Encoding, FontDescriptor, ICCColorProfileMetadata, ImageMetadata, Matrix,
        NumberTree, ObjRef, PageLabel, PdfString, Point, Rectangle, StreamMetadata,
    },
    low::{self, ID},
    lowering::{lower_dict, lower_outline_items, Lowerable, Lowering},
    write::{Formatter, PdfName, Serialize},
    xmp::{self, XmpWriter},
};

mod metadata;
mod page;
pub use metadata::{Info, Metadata};
pub use page::{Page, Resources};

#[derive(Debug, Clone)]
/// Information for the Outline of the document
pub struct Outline {
    /// Immediate children of this item
    pub children: Vec<OutlineItem>,
}

impl Default for Outline {
    fn default() -> Self {
        Self::new()
    }
}

impl Outline {
    /// Creates a new outline struct
    pub fn new() -> Self {
        Self { children: vec![] }
    }
}

/// One item in the outline
#[derive(Debug, Clone)]
pub struct OutlineItem {
    /// The title of the outline item
    pub title: PdfString,
    /// The destination to navigate to
    pub dest: Destination,
    /// Immediate children of this item
    pub children: Vec<OutlineItem>,
}

/// A destination of a GoTo Action
#[derive(Debug, Copy, Clone)]
pub enum Destination {
    /// Scroll to page {0} at height {1} while fitting the page to the viewer
    PageFitH(usize, usize),
}

/// This struct represents a global resource
#[derive(Debug, Copy, Clone)]
pub struct GlobalResource<T> {
    /// The index into the global list
    pub(crate) index: usize,
    /// Marker for contained T
    _phantom: PhantomData<fn() -> T>,
}

impl<T> From<GlobalResource<T>> for Resource<T> {
    fn from(value: GlobalResource<T>) -> Self {
        Resource::Global(value)
    }
}

/// This enum represents a resource of type T for use in a dictionary.
///
/// It does not implement serialize, because it's possible that an index needs to be resolved
#[derive(Debug, Clone)]
pub enum Resource<T> {
    /// Use the resource at {index} from the global list
    Global(GlobalResource<T>),
    /// Use the value in the box
    Immediate(Box<T>),
}

impl<T> Resource<T> {
    /// New global resource reference with the given index
    pub fn global(index: usize) -> Self {
        Self::Global(GlobalResource {
            index,
            _phantom: PhantomData,
        })
    }
}

#[derive(Debug, Clone)]
/// A type 3 font
pub struct Type3Font<'a> {
    /// The name of the font
    pub name: Option<PdfName<'a>>,
    /// The largest boundig box that fits all glyphs
    pub font_bbox: Rectangle<i32>,
    /// Font characteristics
    pub font_descriptor: Option<FontDescriptor<'a>>,
    /// The matrix to map glyph space into text space
    pub font_matrix: Matrix<f32>,
    /// The first used char key
    pub first_char: u8,
    /// The last used char key
    pub last_char: u8,
    /// Dict of char names to drawing procedures
    pub char_procs: Dict<Ascii85Stream<'a>>,
    /// Dict of encoding value to char names
    pub encoding: Encoding<'a>,
    /// Width of every char between first and last (in fontunits, i.e. 1/72000 in)
    pub widths: Vec<u32>,
    /// ToUnicode CMap stream
    pub to_unicode: Option<Resource<Ascii85Stream<'a>>>,
}

impl Default for Type3Font<'_> {
    fn default() -> Self {
        Self {
            font_bbox: Rectangle {
                ll: Point::default(),
                ur: Point::default(),
            },
            name: None,
            font_matrix: Matrix::default_glyph(),
            font_descriptor: None,
            first_char: 0,
            last_char: 255,
            char_procs: Dict::new(),
            encoding: Encoding {
                base_encoding: None,
                differences: None,
            },
            widths: vec![],
            to_unicode: None,
        }
    }
}

#[derive(Debug, Clone)]
/// A Font resource
pub enum Font<'a> {
    /// A type 3 font i.e. arbitrary glyph drawings
    Type3(Type3Font<'a>),
}

/// An embedded object resource
#[derive(Debug)]
pub enum XObject {
    /// An image
    Image(Image),
}

#[derive(Debug)]
/// An Image resource
pub struct Image {
    /// The metadata for this image
    pub meta: ImageMetadata,
    /// The data for the image
    pub data: Vec<u8>,
}

impl From<Image> for XObject {
    fn from(value: Image) -> Self {
        XObject::Image(value)
    }
}

/// A dict of resources
pub type DictResource<T> = Dict<Resource<T>>;
/// A referenced or immediate dict of resources
pub type ResDictRes<T> = Resource<Dict<Resource<T>>>;

/// The global context for lowering
#[derive(Debug, Default)]
pub struct Res<'a> {
    /// Font resources
    pub fonts: Vec<Font<'a>>,
    /// Font dict resources
    pub font_dicts: Vec<DictResource<Font<'a>>>,
    /// Embedded object resources
    pub x_objects: Vec<XObject>,
    /// Embedded object dict resources
    pub x_object_dicts: Vec<DictResource<XObject>>,
    /// Char Procedure resources
    pub char_procs: Vec<Ascii85Stream<'a>>,
    /// Encoding resources
    pub encodings: Vec<Encoding<'a>>,
}

fn push<T>(vec: &mut Vec<T>, value: T) -> usize {
    let index = vec.len();
    vec.push(value);
    index
}

impl<'a> Res<'a> {
    /// Push an XObject, returning the index it was pushed at
    pub fn push_xobject<T: Into<XObject>>(&mut self, value: T) -> GlobalResource<XObject> {
        GlobalResource {
            index: push(&mut self.x_objects, value.into()),
            _phantom: PhantomData,
        }
    }

    /// Push a font dictionary, returning the index it was pushed at
    pub fn push_font(&mut self, value: Font<'a>) -> GlobalResource<Font<'static>> {
        GlobalResource {
            index: push(&mut self.fonts, value),
            _phantom: PhantomData,
        }
    }

    /// Push a font dictionary, returning the index it was pushed at
    pub fn push_font_dict(
        &mut self,
        value: DictResource<Font<'a>>,
    ) -> GlobalResource<DictResource<Font<'static>>> {
        GlobalResource {
            index: push(&mut self.font_dicts, value),
            _phantom: PhantomData,
        }
    }
}

/// An icc based color profile
pub struct ICCBasedColorProfile<'a> {
    /// The data file
    pub stream: &'a [u8],
    /// metadata
    pub meta: ICCColorProfileMetadata,
}

/// High-Level output intent
pub type OutputIntent = common::OutputIntent<ICCBasedColorProfile<'static>>;

/// Entrypoint to the high-level API
///
/// Create a new handle to start creating a PDF document
pub struct Handle<'a> {
    /// The info/metadata
    pub meta: Metadata,
    /// The pages
    pub pages: Vec<Page<'a>>,
    /// The settings for page numbering for a PDF viewer
    pub page_labels: NumberTree<PageLabel>,
    /// The outline for a PDF viewer
    pub outline: Outline,
    /// The global resource struct
    pub res: Res<'a>,
    /// The output intents
    pub output_intents: Vec<OutputIntent>,
}

impl Default for Handle<'_> {
    fn default() -> Self {
        Handle::new()
    }
}

#[derive(Debug, Clone)]
/// A text stream in the PDF
pub struct Ascii85Stream<'a> {
    /// The data of this stream
    pub data: Cow<'a, [u8]>,
    /// The metadata for this stream
    pub meta: StreamMetadata,
}

fn pdf_string_of(o: &Option<String>) -> io::Result<Option<PdfString>> {
    let s = o.as_deref().map(PdfString::from_str).transpose()?;
    Ok(s)
}

fn pdf_list_of(o: &[String]) -> io::Result<Option<PdfString>> {
    if o.is_empty() {
        Ok(None)
    } else {
        let s = PdfString::from_str(&o.join(", "))?;
        Ok(Some(s))
    }
}

impl Handle<'_> {
    /// Creates a new handle
    pub fn new() -> Self {
        Self {
            meta: Metadata::new(),
            res: Res::default(),
            page_labels: NumberTree::new(),
            outline: Outline::new(),
            pages: vec![],
            output_intents: vec![],
        }
    }

    fn prepare_xmp(&self) -> io::Result<Vec<u8>> {
        let now = Local::now().fixed_offset();

        let mut writer = XmpWriter::new(Vec::new())?;
        writer.add_description(&xmp::Pdf {
            producer: self.meta.producer.clone(),
        })?;
        writer.add_description(&xmp::DublinCore {
            title: self
                .meta
                .title
                .as_ref()
                .map(|title| BTreeMap::from([(xmp::Lang::Default, title.clone())]))
                .unwrap_or_default(),
            format: "application/pdf",
            creator: self.meta.author.clone(),
            publisher: self.meta.publisher.clone(),
        })?;
        writer.add_description(&xmp::PdfAId {
            part: 2,
            conformance: 'B',
        })?;
        writer.add_description(&xmp::XmpBasic {
            creator_tool: self.meta.producer.clone(),
            create_date: self.meta.creation_date,
            modify_date: self.meta.modify_date,
            metadata_date: now,
        })?;
        writer.add_description(&xmp::XmpMM {
            document_id: Uuid::new_v4(),
            instance_id: Uuid::new_v4(),
        })?;

        writer.finish()
    }

    /// Write the whole PDF to the given writer
    pub fn write<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        let mut fmt = Formatter::new(w);

        let gen = 0;
        let make_ref = move |id: u64| ObjRef { id, gen };
        let mut lowering = Lowering::new(self);

        // Start
        writeln!(fmt.inner, "%PDF-1.5")?;
        fmt.inner.write_all(&[b'%', 180, 200, 220, 240, b'\n'])?;

        // **OutputIntent**
        let mut output_intents = Vec::with_capacity(self.output_intents.len());
        for high_oi in &self.output_intents {
            let dest_output_profile = match &high_oi.dest_output_profile {
                None => None,
                Some(profile) => {
                    let r = make_ref(lowering.id_gen.next());
                    let stream = low::FlateAscii85Stream {
                        data: profile.stream,
                        meta: StreamMetadata::ColorProfile(profile.meta),
                    };
                    fmt.obj(r, &stream)?;
                    Some(r)
                }
            };

            let low_oi = common::OutputIntent::<ObjRef> {
                subtype: high_oi.subtype,
                dest_output_profile,
                output_condition: high_oi.output_condition.clone(),
                output_condition_identifier: high_oi.output_condition_identifier.clone(),
                registry_name: high_oi.registry_name.clone(),
                info: high_oi.info.clone(),
            };

            let r = make_ref(lowering.id_gen.next());
            fmt.obj(r, &low_oi)?;
            output_intents.push(r);
        }

        // Catalog ID
        let catalog_id = lowering.id_gen.next();

        // **Info**
        let info_id = if self.meta.is_empty() {
            None
        } else {
            let info_id = lowering.id_gen.next();
            let r = make_ref(info_id);

            let info = Info {
                title: pdf_string_of(&self.meta.title)?,
                author: pdf_list_of(&self.meta.author)?,
                subject: pdf_string_of(&self.meta.subject)?,
                keywords: pdf_list_of(&self.meta.keywords)?,
                creator: pdf_string_of(&self.meta.creator)?,
                producer: Some(PdfString::from_str(&self.meta.producer)?),
                creation_date: Some(self.meta.creation_date),
                mod_date: Some(self.meta.modify_date),
                trapped: None,
            };

            fmt.obj(r, &info)?;
            Some(r)
        };

        // **Metadata**
        let meta_id = lowering.id_gen.next();
        let meta_ref = make_ref(meta_id);
        let xmp = low::Stream {
            data: self.prepare_xmp()?,
            meta: StreamMetadata::MetadataXML,
        };
        fmt.obj(meta_ref, &xmp)?;

        // **Pages**
        let mut pages = low::Pages { kids: vec![] };
        let pages_id = lowering.id_gen.next();
        let pages_ref = make_ref(pages_id);

        for page in &self.pages {
            let page_id = lowering.id_gen.next();
            let contents_id = lowering.id_gen.next();
            let contents_ref = make_ref(contents_id);

            let contents = low::Stream {
                data: page.contents.clone(),
                meta: StreamMetadata::None,
            };
            fmt.obj(contents_ref, &contents)?;

            let page_ref = make_ref(page_id);
            let page_low = low::Page {
                parent: pages_ref,
                resources: low::Resources {
                    font: lowering.font_dicts.map_dict(
                        &page.resources.fonts,
                        &mut lowering.fonts,
                        &mut lowering.font_ctx,
                        &mut lowering.id_gen,
                    ),
                    x_object: lowering.x_object_dicts.map_dict(
                        &page.resources.x_objects,
                        &mut lowering.x_objects,
                        &mut (),
                        &mut lowering.id_gen,
                    ),
                    proc_set: &page.resources.proc_sets,
                },
                contents: contents_ref,
                media_box: Some(page.media_box),
            };
            fmt.obj(page_ref, &page_low)?;
            pages.kids.push(page_ref);
        }

        for (font_dict_ref, font_dict) in lowering.font_dicts.store.values() {
            let dict = lower_dict(
                font_dict,
                &mut lowering.fonts,
                &mut lowering.font_ctx,
                &mut lowering.id_gen,
            );
            fmt.obj(*font_dict_ref, &dict)?;
        }

        for (font_ref, font) in lowering.fonts.store.values() {
            let font_low = font.lower(&mut lowering.font_ctx, &mut lowering.id_gen);
            fmt.obj(*font_ref, &font_low)?;
        }

        for (x_ref, x) in lowering.x_objects.store.values() {
            let x_low = x.lower(&mut (), &mut lowering.id_gen);
            fmt.obj(*x_ref, &x_low)?;
        }

        // FIXME: this only works AFTER all fonts are lowered
        for (cproc_ref, char_proc) in lowering.font_ctx.text_streams.store.values() {
            let cp = char_proc.lower(&mut (), &mut lowering.id_gen);
            fmt.obj(*cproc_ref, &cp)?;
        }

        let pages_ref = make_ref(pages_id);
        fmt.obj(pages_ref, &pages)?;

        let pl_ref = if !self.page_labels.is_empty() {
            let page_labels_id = lowering.id_gen.next();
            let page_labels_ref = make_ref(page_labels_id);

            fmt.obj(page_labels_ref, &self.page_labels)?;
            Some(page_labels_ref)
        } else {
            None
        };

        // **Outline**
        let ol_ref = if !self.outline.children.is_empty() {
            let mut ol_acc = Vec::new();
            let outline_ref = make_ref(lowering.id_gen.next());
            let (first, last) = lower_outline_items(
                &mut ol_acc,
                &pages.kids,
                &self.outline.children,
                outline_ref,
                &mut lowering.id_gen,
            )
            .unwrap(); // safe because this will always return Some for non empty children
            let outline = low::Outline {
                first,
                last,
                count: self.outline.children.len(),
            };

            for (r, item) in ol_acc {
                fmt.obj(r, &item)?;
            }

            fmt.obj(outline_ref, &outline)?;
            Some(outline_ref)
        } else {
            None
        };

        // **Catalog**
        let catalog = low::Catalog {
            version: None,
            pages: pages_ref,
            page_labels: pl_ref,
            outline: ol_ref,
            output_intents,
            metadata: Some(meta_ref),
        };
        let catalog_ref = make_ref(catalog_id);
        fmt.obj(catalog_ref, &catalog)?;

        // **xref**
        let startxref = fmt.xref()?;
        let id = self.compute_id(&fmt);

        writeln!(fmt.inner, "trailer")?;

        let trailer = low::Trailer {
            size: fmt.xref.len(),
            root: make_ref(catalog_id),
            info: info_id,
            id,
        };
        trailer.write(&mut fmt)?;

        writeln!(fmt.inner, "startxref")?;
        writeln!(fmt.inner, "{}", startxref)?;
        writeln!(fmt.inner, "%%EOF")?;

        Ok(())
    }

    // Consume for the ID
    fn compute_id(&self, fmt: &Formatter) -> ID {
        let mut id_ctx = md5::Context::new();

        // - The current time
        let now = chrono::Local::now().to_string();
        id_ctx.consume(now);

        // - A string representation of the file’s location, usually a pathname
        // TODO

        // - The size of the file in bytes
        let len = fmt.inner.bytes_written();
        id_ctx.consume(len.to_ne_bytes());

        // - The values of all entries in the file’s document information dictionary
        if let Some(a) = &self.meta.title {
            id_ctx.consume(a.as_bytes());
        }

        for a in &self.meta.author {
            id_ctx.consume(a.as_bytes());
        }

        if let Some(a) = &self.meta.subject {
            id_ctx.consume(a.as_bytes());
        }

        for kw in &self.meta.keywords {
            id_ctx.consume(kw.as_bytes());
        }

        if let Some(a) = &self.meta.creator {
            id_ctx.consume(a.as_bytes());
        }

        id_ctx.consume(self.meta.producer.as_bytes());

        // --------------------------------------------------

        let digest = id_ctx.compute();
        ID {
            original: digest,
            current: digest,
        }
    }
}
