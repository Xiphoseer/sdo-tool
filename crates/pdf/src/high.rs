//! High-Level API

use std::{borrow::Cow, collections::BTreeMap, io, str::FromStr};

use chrono::{DateTime, Local};
use io::Write;
use uuid::Uuid;

use crate::{
    common::{
        self, Dict, Encoding, FontDescriptor, ICCColorProfileMetadata, ImageMetadata, Matrix,
        NumberTree, ObjRef, PageLabel, PdfString, Point, ProcSet, Rectangle, StreamMetadata,
        Trapped,
    },
    low::{self, ID},
    lowering::{lower_dict, lower_outline_items, Lowerable, Lowering},
    write::{Formatter, PdfName, Serialize},
    xmp::{self, XmpWriter},
};

/// A single page
pub struct Page<'a> {
    /// The dimensions of the page
    pub media_box: Rectangle<i32>,
    /// The resource used within the page
    pub resources: Resources<'a>,
    /// The content stream of the page
    pub contents: Vec<u8>,
}

/// The Metadata/Info
#[derive(Debug, Default)]
pub struct Metadata {
    /// The title
    pub title: Option<String>,
    /// The author
    pub author: Vec<String>,
    /// The subject
    pub subject: Option<String>,
    /// A list of keywords
    pub keywords: Vec<String>,
    /// The program used to create the source
    pub creator: Option<String>,
    /// The program used to create the source
    pub publisher: Vec<String>,
    /// The program that produced the file (this library)
    pub producer: String,
}

impl Metadata {
    /// Check whether the info contains any meaningful data
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.author.is_empty()
            && self.subject.is_none()
            && self.keywords.is_empty()
            && self.publisher.is_empty()
            && self.creator.is_none()
            && self.producer.is_empty()
    }
}

/// The Metadata/Info
#[derive(Debug, Default)]
pub struct Info {
    /// The title
    pub title: Option<PdfString>,
    /// The author
    pub author: Option<PdfString>,
    /// The subject
    pub subject: Option<PdfString>,
    /// A list of keywords
    pub keywords: Option<PdfString>,
    /// The program used to create the source
    pub creator: Option<PdfString>,
    /// The program that produced the file (this library)
    pub producer: Option<PdfString>,

    /// The date of creation
    pub creation_date: Option<DateTime<Local>>,
    /// The date of the last modification
    pub mod_date: Option<DateTime<Local>>,

    /// Whether the PDF is *trapped*
    pub trapped: Option<Trapped>,
}

impl Serialize for Info {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        let mut dict = f.pdf_dict();
        if let Some(title) = &self.title {
            dict.field("Title", title)?;
        }
        if let Some(author) = &self.author {
            dict.field("Author", author)?;
        }
        if let Some(subject) = &self.subject {
            dict.field("Subject", subject)?;
        }
        if let Some(keywords) = &self.keywords {
            dict.field("Keywords", keywords)?;
        }
        if let Some(creator) = &self.creator {
            dict.field("Creator", creator)?;
        }
        if let Some(producer) = &self.producer {
            dict.field("Producer", producer)?;
        }

        if let Some(creation_date) = &self.creation_date {
            dict.field("CreationDate", creation_date)?;
        }
        if let Some(mod_date) = &self.mod_date {
            dict.field("ModDate", mod_date)?;
        }
        if let Some(trapped) = &self.trapped {
            dict.field("Trapped", trapped)?;
        }
        dict.finish()?;
        Ok(())
    }
}

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

/// This enum represents a resource of type T for use in a dictionary.
///
/// It does not implement serialize, because it's possible that an index needs to be resolved
#[derive(Debug)]
pub enum Resource<T> {
    /// Use the resource at {index} from the global list
    Global {
        /// The index into the global list
        index: usize,
    },
    /// Use the value in the box
    Immediate(Box<T>),
}

#[derive(Debug)]
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
    /// Width of every char between first and last
    pub widths: Vec<u32>,
    /// ToUnicode CMap stream
    pub to_unicode: Option<Ascii85Stream<'a>>,
}

impl<'a> Default for Type3Font<'a> {
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

#[derive(Debug)]
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

/// A dict of resources
pub type DictResource<T> = Dict<Resource<T>>;
/// A referenced or immediate dict of resources
pub type ResDictRes<T> = Resource<Dict<Resource<T>>>;

/// The resources of a page
pub struct Resources<'a> {
    /// A dict of font resources
    pub fonts: ResDictRes<Font<'a>>,
    /// A dict of embedded object resources
    pub x_objects: ResDictRes<XObject>,
    /// A set of valid procedures
    pub proc_sets: Vec<ProcSet>,
}

impl<'a> Default for Resources<'a> {
    fn default() -> Self {
        Resources {
            fonts: Resource::Immediate(Box::new(Dict::new())),
            x_objects: Resource::Immediate(Box::new(Dict::new())),
            proc_sets: vec![ProcSet::PDF, ProcSet::Text],
        }
    }
}
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
    // /// Encoding resources
    // pub encodings: Vec<Encoding<'a>>,
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

impl<'a> Default for Handle<'a> {
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

impl<'a> Handle<'a> {
    /// Creates a new handle
    pub fn new() -> Self {
        Self {
            meta: Metadata::default(),
            res: Res::default(),
            page_labels: NumberTree::new(),
            outline: Outline::new(),
            pages: vec![],
            output_intents: vec![],
        }
    }

    fn prepare_xmp(&self, create_date: DateTime<Local>) -> io::Result<Vec<u8>> {
        let now = Local::now();

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
            modify_date: create_date,
            create_date,
            metadata_date: now,
        })?;
        writer.add_description(&xmp::XmpMM {
            document_id: Uuid::new_v4(),
            instance_id: Uuid::new_v4(),
        })?;

        writer.finish()
    }

    /// Write the whole PDF to the given writer
    pub fn write<W: Write>(&self, w: &mut W) -> io::Result<()> {
        let mut fmt = Formatter::new(w);

        let gen = 0;
        let make_ref = move |id: u64| ObjRef { id, gen };
        let now = Local::now();
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
                creation_date: Some(now),
                mod_date: Some(now),
                trapped: None,
            };

            fmt.obj(r, &info)?;
            Some(r)
        };

        // **Metadata**
        let meta_id = lowering.id_gen.next();
        let meta_ref = make_ref(meta_id);
        let xmp = low::Stream {
            data: self.prepare_xmp(now)?,
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

        for a in &self.meta.creator {
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
