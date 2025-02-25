//! Low-Level API
//!
//! This module contains structs and enums for representing/creating a PDF
//! that is already split up into objects with opaque reference IDs.

use std::{
    borrow::Cow,
    io::{self, Write},
};

use flate2::{write::ZlibEncoder, Compression};

use crate::{
    common::{
        Dict, Encoding, FontDescriptor, Matrix, ObjRef, PdfString, ProcSet, Rectangle,
        StreamMetadata,
    },
    encoding::ascii_85_encode,
    write::{Formatter, PdfName, Serialize},
};

/// Destination of a GoTo action
#[derive(Debug, Clone)]
pub enum Destination {
    /// Page @0, fit the page into view and scroll to height {1}
    PageFitH(ObjRef, usize),
}

/// A PDF action
#[derive(Debug, Clone)]
pub enum Action {
    /// Go to some destination within the document
    GoTo(Destination),
}

impl Serialize for Destination {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        match self {
            Self::PageFitH(r, top) => f
                .pdf_arr()
                .entry(r)?
                .entry(&PdfName("FitH"))?
                .entry(top)?
                .finish(),
        }
    }
}

/// The root outline item
#[derive(Debug, Clone)]
pub struct Outline {
    /// The first item
    pub first: ObjRef,
    /// The last item
    pub last: ObjRef,
    /// The total amount of items
    pub count: usize,
}

impl Serialize for Outline {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_dict()
            .field("Type", &PdfName("Outline"))?
            .field("First", &self.first)?
            .field("Last", &self.last)?
            .field("Count", &self.count)?
            .finish()
    }
}

/// A child outline item
#[derive(Debug, Clone)]
pub struct OutlineItem {
    /// The title of the outline item
    pub title: PdfString,
    /// The parent of this item
    pub parent: ObjRef,
    /// The previous siblig
    pub prev: Option<ObjRef>,
    /// The next sibling
    pub next: Option<ObjRef>,
    /// The first child
    pub first: Option<ObjRef>,
    /// The last child
    pub last: Option<ObjRef>,
    /// The total amount of children
    pub count: usize,
    /// The destination to be used
    pub action: Action,
}

impl Serialize for OutlineItem {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        let mut dict = f.pdf_dict();
        dict.field("Title", &self.title)?
            .field("Parent", &self.parent)?
            .opt_field("Prev", &self.prev)?
            .opt_field("Next", &self.next)?
            .opt_field("First", &self.first)?
            .opt_field("Last", &self.last)?
            .field("Count", &self.count)?;
        match &self.action {
            Action::GoTo(dest) => dict.field("Dest", dest),
        }?;
        dict.finish()
    }
}

/// A page object
pub struct Page<'a> {
    /// Reference to the parent
    pub parent: ObjRef,
    /// The content stream of the page
    pub contents: ObjRef,
    /// The resources of this page
    pub resources: Resources<'a>,
    /// (required, inheritable) describes the bound of the physical page
    /// in default user units
    pub media_box: Option<Rectangle<i32>>,
}

impl Serialize for Page<'_> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_dict()
            .field("Type", &PdfName("Page"))?
            .field("Parent", &self.parent)?
            .opt_field("MediaBox", &self.media_box)?
            .field("Resources", &self.resources)?
            .field("Contents", &self.contents)?
            .finish()
    }
}

/// A resource entry
pub enum Resource<T> {
    /// Reference to another object
    Ref(ObjRef),
    /// A resource that is serialized in place
    Immediate(T),
}

impl<T: Serialize> Serialize for Resource<T> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        match self {
            Self::Ref(r) => r.write(f),
            Self::Immediate(value) => value.write(f),
        }
    }
}

/// A type 3 font resource
pub struct Type3Font<'a> {
    /// The name of the object
    pub name: Option<PdfName<'a>>,
    /// The largest boundig box that fits all glyphs
    pub font_bbox: Rectangle<i32>,
    /// The matrix to map glyph space into text space
    pub font_matrix: Matrix<f32>,
    /// The first used char key
    pub first_char: u8,
    /// The last used char key
    pub last_char: u8,
    /// Dict of encoding value to char names
    pub encoding: Resource<Encoding<'a>>,
    /// Dict of char names to drawing procedures
    pub char_procs: Dict<ObjRef>,
    /// Width of every char between first and last
    pub widths: &'a [u32],
    /// Font characteristics
    pub font_descriptor: Option<FontDescriptor<'a>>,
    /// Optional reference to a CMap stream
    pub to_unicode: Option<ObjRef>,
}

/// A font resource
pub enum Font<'a> {
    /// A type 3 font resource
    Type3(Type3Font<'a>),
}

impl Serialize for Font<'_> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        let mut dict = f.pdf_dict();
        dict.field("Type", &PdfName("Font"))?;
        match self {
            Self::Type3(font) => {
                dict.field("Subtype", &PdfName("Type3"))?
                    .opt_field("BaseFont", &font.name)?
                    .field("FontBBox", &font.font_bbox)?
                    .field("FontMatrix", &font.font_matrix)?
                    .field("FirstChar", &font.first_char)?
                    .field("LastChar", &font.last_char)?
                    .field("Encoding", &font.encoding)?
                    .field("CharProcs", &font.char_procs)?
                    .arr_field("Widths", font.widths)?
                    .opt_field("FontDescriptor", &font.font_descriptor)?
                    .opt_field("ToUnicode", &font.to_unicode)?;
            }
        }
        dict.finish()?;
        Ok(())
    }
}

/// A character drawing procedure
pub struct Ascii85Stream<'a> {
    /// The data of this stream
    pub data: Cow<'a, [u8]>,
    /// The associated metadata
    pub meta: StreamMetadata,
}

impl Serialize for Ascii85Stream<'_> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        let mut buf = Vec::new();
        let len = ascii_85_encode(self.data.as_ref(), &mut buf)?;
        buf.push(10);
        f.pdf_dict()
            .embed(&self.meta)?
            .field("Length", &len)?
            .field("Filter", &PdfName("ASCII85Decode"))?
            .finish()?;
        f.pdf_stream(&buf)?;
        Ok(())
    }
}

/// A character drawing procedure
pub struct FlateStream<'a> {
    /// The data of this stream
    pub data: Cow<'a, [u8]>,
    /// The associated metadata
    pub meta: StreamMetadata,
}

impl Serialize for FlateStream<'_> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        let mut e = ZlibEncoder::new(Vec::new(), Compression::best());
        e.write_all(self.data.as_ref()).unwrap();
        let mut buf = e.finish()?;
        let len = buf.len();
        buf.push(10);
        f.pdf_dict()
            .embed(&self.meta)?
            .field("Length", &len)?
            .field("Filter", &PdfName("FlateDecode"))?
            .finish()?;
        f.pdf_stream(&buf)?;
        Ok(())
    }
}

/// A character drawing procedure
pub struct FlateAscii85Stream<'a> {
    /// The data of this stream
    pub data: &'a [u8],
    /// The associated metadata
    pub meta: StreamMetadata,
}

impl Serialize for FlateAscii85Stream<'_> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        let mut e = ZlibEncoder::new(Vec::new(), Compression::best());
        e.write_all(self.data.as_ref()).unwrap();
        let mut buf = e.finish()?;
        let mut out = Vec::new();
        let len = ascii_85_encode(&buf, &mut out)?;
        buf.push(10);
        f.pdf_dict()
            .embed(&self.meta)?
            .field("Length", &len)?
            .field(
                "Filter",
                &(PdfName("ASCII85Decode"), PdfName("FlateDecode")),
            )?
            .finish()?;
        f.pdf_stream(&out)?;
        Ok(())
    }
}

/// An emedded object resource
pub enum XObject<'a> {
    /// An image object
    Image(Ascii85Stream<'a>),
}

impl Serialize for XObject<'_> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        match self {
            Self::Image(i) => i.write(f),
        }
    }
}

/// A dict of resources
pub type DictResource<T> = Dict<Resource<T>>;
/// A resource of a dictionary
pub type ResDict<T> = Resource<Dict<T>>;
/// A referenced or immediate dict of resources
pub type ResDictRes<T> = ResDict<Resource<T>>;

/// The resources of a page
pub struct Resources<'a> {
    /// A dict of font resources
    pub font: ResDictRes<Font<'a>>,
    /// A dict of embedded object resources
    pub x_object: ResDict<ObjRef>,
    /// A set of valid procedures
    pub proc_set: &'a [ProcSet],
}

impl Serialize for Resources<'_> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_dict()
            .dict_res_field("Font", &self.font)?
            .dict_res_field("XObject", &self.x_object)?
            .arr_field("ProcSet", self.proc_set)?
            .finish()
    }
}

/// The list of pages
pub struct Pages {
    /// References to the individual pages
    pub kids: Vec<ObjRef>,
}

impl Serialize for Pages {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_dict()
            .field("Type", &PdfName("Pages"))?
            .field("Count", &self.kids.len())?
            .field("Kids", &self.kids)?
            .finish()
    }
}

/// A data stream
pub struct Stream {
    /// The (unencoded) data
    pub data: Vec<u8>,
    /// Additional metadata
    pub meta: StreamMetadata,
}

impl Serialize for Stream {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        let mut len = self.data.len();
        if self.data.ends_with(&[0x0a]) {
            len -= 1;
        }
        f.pdf_dict()
            .embed(&self.meta)?
            .field("Length", &len)?
            .finish()?;
        f.pdf_stream(&self.data)?;
        Ok(())
    }
}

/// Well-known PDF Versions
pub enum PdfVersion {
    /// PDF-1.0
    V1_0,
    /// PDF-1.1
    V1_1,
    /// PDF-1.2
    V1_2,
    /// PDF-1.3
    V1_3,
    /// PDF-1.4
    V1_4,
    /// PDF-1.5
    V1_5,
    /// PDF-1.6
    V1_6,
    /// PDF-1.7
    V1_7,
}

impl Serialize for PdfVersion {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        match self {
            Self::V1_0 => PdfName("1.0").write(f),
            Self::V1_1 => PdfName("1.1").write(f),
            Self::V1_2 => PdfName("1.2").write(f),
            Self::V1_3 => PdfName("1.3").write(f),
            Self::V1_4 => PdfName("1.4").write(f),
            Self::V1_5 => PdfName("1.5").write(f),
            Self::V1_6 => PdfName("1.6").write(f),
            Self::V1_7 => PdfName("1.7").write(f),
        }
    }
}

/// The catalog/root of the document
pub struct Catalog {
    /// The PDF Version
    pub version: Option<PdfVersion>,
    // Extensions
    /// Reference to the list of pages
    pub pages: ObjRef,
    /// Optional reference to the page labels
    pub page_labels: Option<ObjRef>,
    /// Optional reference to the outline
    pub outline: Option<ObjRef>,
    /// Optional List of output intents
    pub output_intents: Vec<ObjRef>,
    /// XMP metadata stream
    pub metadata: Option<ObjRef>,
}

impl Serialize for Catalog {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_dict()
            .field("Type", &PdfName("Catalog"))?
            .opt_field("Version", &self.version)?
            .field("Pages", &self.pages)?
            .opt_field("PageLabels", &self.page_labels)?
            .opt_field("Outlines", &self.outline)?
            .opt_arr_field("OutputIntents", &self.output_intents)?
            .opt_field("Metadata", &self.metadata)?
            .finish()
    }
}

/// The structure that holds the document IDs.
#[allow(clippy::upper_case_acronyms)]
pub struct ID {
    /// The ID for the original (gen 0) document
    pub original: md5::Digest,
    /// The ID for the current generation of the document
    pub current: md5::Digest,
}

impl Serialize for ID {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_arr()
            .entry(&self.original)?
            .entry(&self.current)?
            .finish()
    }
}

/// The trailer of the document
pub struct Trailer {
    /// The size of the document / number of objects
    pub size: usize,
    /// Optional reference to the info struct
    pub info: Option<ObjRef>,
    /// Refernce to the root/catalog
    pub root: ObjRef,
    /// The ID String
    pub id: ID,
}

impl Serialize for Trailer {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_dict()
            .field("Size", &self.size)?
            .opt_field("Info", &self.info)?
            .field("Root", &self.root)?
            .field("ID", &self.id)?
            .finish()
    }
}
