use std::{borrow::Cow, io};

use chrono::{DateTime, Local};
use io::Write;
use pdf::{object::PlainRef, primitive::PdfString};

use crate::{
    common::{Dict, Encoding, Matrix, NumberTree, PageLabel, Point, ProcSet, Rectangle, Trapped},
    low,
    lowering::{lower_dict, lower_outline_items, Lowerable, Lowering},
    write::{Formatter, PdfName, Serialize},
};

pub struct Page<'a> {
    pub media_box: Rectangle<i32>,
    pub resources: Resources<'a>,
    pub contents: String,
}

#[derive(Debug, Default)]
pub struct Info {
    pub title: Option<PdfString>,
    pub author: Option<PdfString>,
    pub subject: Option<PdfString>,
    pub keywords: Option<PdfString>,
    pub creator: Option<PdfString>,
    pub producer: Option<PdfString>,

    pub creation_date: Option<DateTime<Local>>,
    pub mod_date: Option<DateTime<Local>>,

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

impl Info {
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.author.is_none()
            && self.subject.is_none()
            && self.keywords.is_none()
            && self.creator.is_none()
            && self.producer.is_none()
            && self.creation_date.is_none()
            && self.mod_date.is_none()
            && self.trapped.is_none()
    }
}

#[derive(Debug, Clone)]
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
    pub fn new() -> Self {
        Self { children: vec![] }
    }
}

#[derive(Debug, Clone)]
pub struct OutlineItem {
    /// The title of the outline item
    pub title: PdfString,
    /// The destination to navigate to
    pub dest: Destination,
    /// Immediate children of this item
    pub children: Vec<OutlineItem>,
}

#[derive(Debug, Copy, Clone)]
pub enum Destination {
    PageFitH(usize, usize),
}

/// This enum represents a resource of type T for use in a dictionary.
///
/// It does not implement serialize, because it's possible that an index needs to be resolved
#[derive(Debug)]
pub enum Resource<T> {
    Global { index: usize },
    Immediate(Box<T>),
}

#[derive(Debug)]
pub struct Type3Font<'a> {
    pub name: Option<PdfName<'a>>,
    pub font_bbox: Rectangle<i32>,
    pub font_matrix: Matrix<f32>,
    pub first_char: u8,
    pub last_char: u8,
    pub char_procs: Dict<CharProc<'a>>,
    pub encoding: Encoding<'a>,
    pub widths: Vec<u32>,
    pub to_unicode: (),
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
            first_char: 0,
            last_char: 255,
            char_procs: Dict::new(),
            encoding: Encoding {
                base_encoding: None,
                differences: None,
            },
            widths: vec![],
            to_unicode: (),
        }
    }
}

#[derive(Debug)]
pub enum Font<'a> {
    Type3(Type3Font<'a>),
}

#[derive(Debug)]
pub enum XObject {}

pub type DictResource<T> = Dict<Resource<T>>;
pub type ResDictRes<T> = Resource<Dict<Resource<T>>>;

pub struct Resources<'a> {
    pub fonts: ResDictRes<Font<'a>>,
    pub x_objects: ResDictRes<XObject>,
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
#[derive(Debug)]
pub struct Res<'a> {
    pub fonts: Vec<Font<'a>>,
    pub font_dicts: Vec<DictResource<Font<'a>>>,
    pub x_objects: Vec<XObject>,
    pub x_object_dicts: Vec<DictResource<XObject>>,
    pub char_procs: Vec<CharProc<'a>>,
    pub encodings: Vec<Encoding<'a>>,
}

impl<'a> Default for Res<'a> {
    fn default() -> Self {
        Self {
            fonts: Vec::new(),
            font_dicts: Vec::new(),
            x_objects: Vec::new(),
            x_object_dicts: Vec::new(),
            char_procs: Vec::new(),
            encodings: Vec::new(),
        }
    }
}

pub struct Handle<'a> {
    pub info: Info,
    pub pages: Vec<Page<'a>>,
    pub page_labels: NumberTree<PageLabel>,
    pub outline: Outline,
    pub res: Res<'a>,
}

impl<'a> Default for Handle<'a> {
    fn default() -> Self {
        Handle::new()
    }
}

#[derive(Debug, Clone)]
pub struct CharProc<'a>(pub Cow<'a, [u8]>);

impl<'a> Handle<'a> {
    pub fn new() -> Self {
        Self {
            info: Info::default(),
            res: Res::default(),
            page_labels: NumberTree::new(),
            outline: Outline::new(),
            pages: vec![],
        }
    }

    pub fn write<W: Write>(&self, w: &mut W) -> io::Result<()> {
        let mut fmt = Formatter::new(w);
        //let mut id_gen = NextID::new(1);

        let gen = 0;
        let make_ref = move |id: u64| PlainRef { id, gen };

        writeln!(fmt.inner, "%PDF-1.5")?;
        writeln!(fmt.inner)?;

        let mut lowering = Lowering::new(self);

        let catalog_id = lowering.id_gen.next();
        let info_id = if self.info.is_empty() {
            None
        } else {
            let info_id = lowering.id_gen.next();
            let r = make_ref(info_id);
            fmt.obj(r, &self.info)?;
            Some(r)
        };

        let mut pages = low::Pages { kids: vec![] };
        let pages_id = lowering.id_gen.next();
        let pages_ref = make_ref(pages_id);

        for page in &self.pages {
            let page_id = lowering.id_gen.next();
            let contents_id = lowering.id_gen.next();
            let contents_ref = make_ref(contents_id);

            let contents = low::Stream {
                data: page.contents.as_bytes().to_vec(),
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

        for (font_dict_ref, font_dict) in &lowering.font_dicts.store {
            let dict = lower_dict(
                font_dict,
                &mut lowering.fonts,
                &mut lowering.font_ctx,
                &mut lowering.id_gen,
            );
            fmt.obj(*font_dict_ref, &dict)?;
        }

        for (font_ref, font) in &lowering.fonts.store {
            let font_low = font.lower(&mut lowering.font_ctx, &mut lowering.id_gen);
            fmt.obj(*font_ref, &font_low)?;
        }

        // FIXME: this only works AFTER all fonts are lowered
        for (cproc_ref, char_proc) in &lowering.font_ctx.0.store {
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

        let catalog = low::Catalog {
            version: None,
            pages: pages_ref,
            page_labels: pl_ref,
            outline: ol_ref,
        };
        let catalog_ref = make_ref(catalog_id);
        fmt.obj(catalog_ref, &catalog)?;

        let startxref = fmt.xref()?;

        writeln!(fmt.inner, "trailer")?;

        let trailer = low::Trailer {
            size: fmt.xref.len(),
            root: make_ref(catalog_id),
            info: info_id,
        };
        trailer.write(&mut fmt)?;

        writeln!(fmt.inner, "startxref")?;
        writeln!(fmt.inner, "{}", startxref)?;
        writeln!(fmt.inner, "%%EOF")?;

        Ok(())
    }
}
