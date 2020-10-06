use std::{borrow::Cow, io};

use pdf::{object::PlainRef, primitive::PdfString};

use crate::{
    common::Dict, common::Encoding, common::Matrix, common::ProcSet, common::Rectangle,
    encoding::ascii_85_encode, write::Formatter, write::PdfName, write::Serialize,
};

#[derive(Debug, Clone)]
pub enum Destination {
    PageFitH(PlainRef, usize),
}

#[derive(Debug, Clone)]
pub enum Action {
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

#[derive(Debug, Clone)]
pub struct Outline {
    /// The first item
    pub first: PlainRef,
    /// The last item
    pub last: PlainRef,
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

#[derive(Debug, Clone)]
pub struct OutlineItem {
    /// The title of the outline item
    pub title: PdfString,
    /// The parent of this item
    pub parent: PlainRef,
    /// The previous siblig
    pub prev: Option<PlainRef>,
    /// The next sibling
    pub next: Option<PlainRef>,
    /// The first child
    pub first: Option<PlainRef>,
    /// The last child
    pub last: Option<PlainRef>,
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

pub struct Page<'a> {
    /// Reference to the parent
    pub parent: PlainRef,
    /// The content stream of the page
    pub contents: PlainRef,
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

pub enum Resource<T> {
    Ref(PlainRef),
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

pub struct Type3Font<'a> {
    pub name: Option<PdfName<'a>>,
    pub font_bbox: Rectangle<i32>,
    pub font_matrix: Matrix<f32>,
    pub first_char: u8,
    pub last_char: u8,
    pub encoding: Resource<Encoding<'a>>,
    pub char_procs: Dict<PlainRef>,
    pub widths: &'a [u32],
}

pub enum Font<'a> {
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
                    .arr_field("Widths", &font.widths)?;
            }
        }
        dict.finish()?;
        Ok(())
    }
}

pub struct CharProc<'a>(pub Cow<'a, [u8]>);

impl<'a> Serialize for CharProc<'a> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_dict()
            .field("Length", &self.0.len())?
            .field("Filter", &PdfName("ASCII85Decode"))?
            .finish()?;
        let mut buf = Vec::new();
        ascii_85_encode(self.0.as_ref(), &mut buf)?;
        buf.push(10);
        f.pdf_stream(&buf)?;
        Ok(())
    }
}

pub enum XObject {
    Image {},
}

impl Serialize for XObject {
    fn write(&self, _f: &mut Formatter) -> io::Result<()> {
        todo!()
    }
}

pub type DictResource<T> = Dict<Resource<T>>;
pub type ResDictRes<T> = Resource<Dict<Resource<T>>>;

pub struct Resources<'a> {
    pub font: ResDictRes<Font<'a>>,
    pub x_object: ResDictRes<XObject>,
    pub proc_set: &'a [ProcSet],
}

impl Serialize for Resources<'_> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_dict()
            .dict_res_field("Font", &self.font)?
            .dict_res_field("XObject", &self.x_object)?
            .arr_field("ProcSet", &self.proc_set)?
            .finish()
    }
}

pub struct Pages {
    pub kids: Vec<PlainRef>,
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

pub struct Stream {
    pub data: Vec<u8>,
}

impl Serialize for Stream {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_dict().field("Length", &self.data.len())?.finish()?;
        f.pdf_stream(&self.data)?;
        Ok(())
    }
}

pub enum PdfVersion {
    V1_0,
    V1_1,
    V1_2,
    V1_3,
    V1_4,
    V1_5,
    V1_6,
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

pub struct Catalog {
    pub version: Option<PdfVersion>,
    // Extensions
    pub pages: PlainRef,
    pub page_labels: Option<PlainRef>,
    pub outline: Option<PlainRef>,
}

impl Serialize for Catalog {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_dict()
            .field("Type", &PdfName("Catalog"))?
            .opt_field("Version", &self.version)?
            .field("Pages", &self.pages)?
            .opt_field("PageLabels", &self.page_labels)?
            .opt_field("Outlines", &self.outline)?
            .finish()
    }
}

pub struct Trailer {
    pub size: usize,
    pub info: Option<PlainRef>,
    pub root: PlainRef,
}

impl Serialize for Trailer {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_dict()
            .field("Size", &self.size)?
            .opt_field("Info", &self.info)?
            .field("Root", &self.root)?
            .finish()
    }
}
