//! Helpers to turn *high* types into *low* types

use std::{borrow::Cow, collections::HashMap};

use crate::{
    common::Encoding,
    common::{Dict, ObjRef, StreamMetadata},
    high::Ascii85Stream,
    high::Destination,
    high::DictResource,
    high::Font,
    high::Handle,
    high::OutlineItem,
    high::Resource,
    high::XObject,
    low,
    util::NextId,
};

/// Make a ObjRef for an original document (generation 0)
pub fn make_ref(id: u64) -> ObjRef {
    ObjRef { id, gen: 0 }
}

fn lower_dest(pages: &[ObjRef], dest: Destination) -> low::Action {
    use low::Action::*;
    use low::Destination::*;
    match dest {
        Destination::PageFitH(a, top) => {
            let page = pages[a];
            GoTo(PageFitH(page, top))
        }
    }
}

pub(super) fn lower_outline_items(
    acc: &mut Vec<(ObjRef, low::OutlineItem)>,
    pages: &[ObjRef],
    items: &[OutlineItem],
    parent: ObjRef,
    id_gen: &mut NextId,
) -> Option<(ObjRef, ObjRef)> {
    if let Some((last, rest)) = items.split_last() {
        let mut prev = None;
        let first_ref = make_ref(id_gen.next());
        let mut curr = first_ref;

        // most items
        for item in rest {
            let (fc, lc) = match lower_outline_items(acc, pages, &item.children, parent, id_gen) {
                Some((fc, lc)) => (Some(fc), Some(lc)),
                None => (None, None),
            };
            let action = lower_dest(pages, item.dest);
            let next = make_ref(id_gen.next());
            acc.push((
                curr,
                low::OutlineItem {
                    title: item.title.clone(),
                    parent,
                    prev,
                    next: Some(next),
                    first: fc,
                    last: lc,
                    count: 0,
                    action,
                },
            ));
            prev = Some(curr);
            curr = next;
        }

        // Last item
        let (fc, lc) = match lower_outline_items(acc, pages, &last.children, parent, id_gen) {
            Some((fc, lc)) => (Some(fc), Some(lc)),
            None => (None, None),
        };
        let action = lower_dest(pages, last.dest);
        acc.push((
            curr,
            low::OutlineItem {
                title: last.title.clone(),
                parent,
                prev,
                next: None,
                first: fc,
                last: lc,
                count: 0,
                action,
            },
        ));
        Some((first_ref, curr))
    } else {
        None
    }
}

pub(crate) trait Lowerable<'a> {
    type Lower;

    fn lower(&'a self, ctx: &mut LoweringContext<'a>) -> Self::Lower;
}

pub(crate) trait Named {
    fn name() -> &'static str;
}

pub(crate) struct LowerFontCtx<'a> {
    //pub text_streams: LowerBox<'a, Ascii85Stream<'a>>,
    pub encodings: LowerBox<'a, Encoding<'a>>,
    pub text_stream_dicts: LowerBox<'a, Dict<Ascii85Stream<'a>>>,
}

impl<'a, T: Lowerable<'a>> Lowerable<'a> for Dict<T> {
    type Lower = Dict<T::Lower>;

    fn lower(&'a self, ctx: &mut LoweringContext<'a>) -> Self::Lower {
        self.iter()
            .map(|(key, proc)| (key.clone(), proc.lower(ctx)))
            .collect()
    }
}

impl<'a, T: Lowerable<'a>> Lowerable<'a> for Option<T> {
    type Lower = Option<T::Lower>;

    fn lower(&'a self, ctx: &mut LoweringContext<'a>) -> Self::Lower {
        self.as_ref().map(|v| v.lower(ctx))
    }
}

impl<'a, T> Lowerable<'a> for Resource<T>
where
    T: Lowerable<'a> + 'a,
    T: Global<'a>,
{
    type Lower = low::Resource<T::Lower>;

    fn lower(&'a self, ctx: &mut LoweringContext<'a>) -> Self::Lower {
        match self {
            Resource::Global(ri) => low::Resource::Ref(T::lookup(ctx, ri.get())),
            Resource::Immediate(v) => low::Resource::Immediate(v.lower(ctx)),
        }
    }
}

impl<'a> Lowerable<'a> for Font<'a> {
    type Lower = low::Font<'a>;

    fn lower(&'a self, ctx: &mut LoweringContext<'a>) -> Self::Lower {
        match self {
            Font::Type3(font) => {
                let char_procs = font.char_procs.lower(ctx);
                let to_unicode = font.to_unicode.lower(ctx);

                low::Font::Type3(low::Type3Font {
                    name: font.name.as_deref(),
                    font_bbox: font.font_bbox,
                    font_descriptor: font.font_descriptor.clone(),
                    font_matrix: font.font_matrix,
                    first_char: font.first_char,
                    last_char: font.last_char,
                    encoding: font.encoding.lower(ctx),
                    char_procs,
                    widths: &font.widths,
                    to_unicode,
                })
            }
        }
    }
}

impl Named for Font<'_> {
    fn name() -> &'static str {
        "Font"
    }
}

impl<'a> Lowerable<'a> for XObject {
    type Lower = low::XObject<'a>;

    fn lower(&'a self, _ctx: &mut LoweringContext) -> Self::Lower {
        match self {
            Self::Image(i) => low::XObject::Image(low::Ascii85Stream {
                data: Cow::Borrowed(&i.data),
                meta: StreamMetadata::Image(i.meta),
            }),
        }
    }
}

impl<'a> Lowerable<'a> for Ascii85Stream<'a> {
    type Lower = ObjRef;

    fn lower(&self, ctx: &mut LoweringContext<'a>) -> Self::Lower {
        let obj_ref = make_ref(ctx.id_gen.next());
        ctx.ascii_85_streams.push((
            obj_ref,
            low::Ascii85Stream {
                data: self.data.clone(),
                meta: self.meta,
            },
        ));
        obj_ref
    }
}

impl<'a> Lowerable<'a> for Encoding<'a> {
    type Lower = Encoding<'a>;

    fn lower(&self, _ctx: &mut LoweringContext) -> Self::Lower {
        self.clone()
    }
}

pub(crate) struct LowerBox<'a, T> {
    name: &'static str,
    pub store: HashMap<usize, (ObjRef, &'a T)>,
    res: &'a [T],
    //next: usize,
}

impl<'a, T> LowerBox<'a, T> {
    fn new(res: &'a [T], name: &'static str) -> Self {
        LowerBox {
            name,
            store: HashMap::new(),
            res,
            //next: res.len(),
        }
    }

    fn lookup(&mut self, index: usize, id_gen: &mut NextId) -> ObjRef {
        if let Some((r, _)) = self.store.get(&index) {
            *r
        } else if let Some(font_dict) = self.res.get(index) {
            let id = id_gen.next();
            let r = make_ref(id);
            self.store.insert(index, (r, font_dict));
            r
        } else {
            panic!("Couldn't find {} Dict #{}", self.name, index);
        }
    }
}

/*pub(crate) fn lower_dict<'a, T: Lowerable<'a>>(
    dict: &'a DictResource<T>,
    ctx: &mut LoweringContext<'a>,
) -> low::DictResource<T::Lower> {
    dict.iter()
        .map(|(key, res)| (key.clone(), inner.map(res, ctx)))
        .collect()
}*/

/*impl<'a, C, T> LowerBox<'a, DictResource<T>, C>
where
    T: Lowerable<'a, C>,
    C: AsMut<LowerBox<'a, DictResource<T>, C>>,
    C: AsMut<LowerBox<'a, T, C>>,
{
    pub fn map_dict(
        &mut self,
        res: &'a ResDictRes<T>,
        inner: &mut LowerBox<'a, T, C>,
        ctx: &mut C,
        id_gen: &mut NextId,
    ) -> low::ResDictRes<T::Lower> {
        res.lower(ctx, id_gen)
        /*match res {
            Resource::Global { index } => {
                if let Some((r, _)) = self.store.get(index) {
                    low::Resource::Ref(*r)
                } else if let Some(font_dict) = self.res.get(*index) {
                    let id = id_gen.next();
                    let r = make_ref(id);
                    self.store.insert(*index, (r, font_dict));
                    low::Resource::Ref(r)
                } else {
                    panic!("Couldn't find {} Dict #{}", self.name, index);
                }
            }
            Resource::Immediate(fonts) => {
                let dict = lower_dict(fonts.as_ref(), inner, ctx, id_gen);
                low::Resource::Immediate(dict)
            }
        }*/
    }
}*/

/*impl<'a, T> LowerBox<'a, T>
where
    T: Lowerable<'a>,
{
    fn put(&mut self, val: &'a T, id_gen: &mut NextId) -> ObjRef {
        let id = id_gen.next();
        let r = make_ref(id);
        let index = self.next;
        self.next += 1;
        self.store.insert(index, (r, val));
        r
    }
}*/

pub(crate) struct LoweringContext<'a> {
    pub id_gen: NextId,
    pub x_objects: LowerXObjectCtx<'a>,
    pub fonts_ctx: LowerFontsCtx<'a>,
    pub font_ctx: LowerFontCtx<'a>,

    /// The ascii 85 streams that need to be written
    pub ascii_85_streams: Vec<(ObjRef, low::Ascii85Stream<'a>)>,
}

pub(crate) struct LowerXObjectCtx<'a> {
    pub x_objects: LowerBox<'a, XObject>,
    pub x_object_dicts: LowerBox<'a, DictResource<XObject>>,
}

pub(crate) struct LowerFontsCtx<'a> {
    pub fonts: LowerBox<'a, Font<'a>>,
    pub font_dicts: LowerBox<'a, DictResource<Font<'a>>>,
}

impl<'a> LoweringContext<'a> {
    pub fn new(doc: &'a Handle) -> Self {
        LoweringContext {
            id_gen: NextId::new(1),
            x_objects: LowerXObjectCtx {
                x_objects: LowerBox::new(&doc.res.x_objects, "XObject"),
                x_object_dicts: LowerBox::new(&doc.res.x_object_dicts, "XObject dictionary"),
            },
            fonts_ctx: LowerFontsCtx {
                fonts: LowerBox::new(&doc.res.fonts, "Font"),
                font_dicts: LowerBox::new(&doc.res.font_dicts, "Font dictionary"),
            },
            font_ctx: LowerFontCtx {
                //text_streams: LowerBox::new(&doc.res.char_procs, "CharProc"),
                encodings: LowerBox::new(&doc.res.encodings, "Encoding"),
                text_stream_dicts: LowerBox::new(&doc.res.char_procs_dicts, "CharProc dictionary"),
            },
            ascii_85_streams: Vec::new(),
        }
    }
}

impl<'a> AsMut<LowerBox<'a, XObject>> for LoweringContext<'a> {
    fn as_mut(&mut self) -> &mut LowerBox<'a, XObject> {
        &mut self.x_objects.x_objects
    }
}

trait Global<'a>: Sized {
    fn lookup(ctx: &mut LoweringContext<'a>, index: usize) -> ObjRef;
}

impl<'a> Global<'a> for XObject {
    fn lookup(ctx: &mut LoweringContext<'a>, index: usize) -> ObjRef {
        ctx.x_objects.x_objects.lookup(index, &mut ctx.id_gen)
    }
}

impl<'a> Global<'a> for DictResource<XObject> {
    fn lookup(ctx: &mut LoweringContext<'a>, index: usize) -> ObjRef {
        ctx.x_objects.x_object_dicts.lookup(index, &mut ctx.id_gen)
    }
}

impl<'a> Global<'a> for Font<'a> {
    fn lookup(ctx: &mut LoweringContext<'a>, index: usize) -> ObjRef {
        ctx.fonts_ctx.fonts.lookup(index, &mut ctx.id_gen)
    }
}

impl<'a> Global<'a> for DictResource<Font<'a>> {
    fn lookup(ctx: &mut LoweringContext<'a>, index: usize) -> ObjRef {
        ctx.fonts_ctx.font_dicts.lookup(index, &mut ctx.id_gen)
    }
}

impl<'a> Global<'a> for Dict<Ascii85Stream<'a>> {
    fn lookup(ctx: &mut LoweringContext<'a>, index: usize) -> ObjRef {
        ctx.font_ctx
            .text_stream_dicts
            .lookup(index, &mut ctx.id_gen)
    }
}

impl<'a> Global<'a> for Encoding<'a> {
    fn lookup(ctx: &mut LoweringContext<'a>, index: usize) -> ObjRef {
        ctx.font_ctx.encodings.lookup(index, &mut ctx.id_gen)
    }
}
