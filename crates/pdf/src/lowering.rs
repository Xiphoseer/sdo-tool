//! Helpers to turn *high* types into *low* types

use std::{borrow::Cow, collections::HashMap};

use crate::{
    common::{Encoding, ObjRef, StreamMetadata},
    high::{
        Ascii85Stream, Destination, DictResource, Font, GlobalResource, Handle, OutlineItem,
        ResDictRes, Resource, XObject,
    },
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
    type Ctx;

    fn lower(&'a self, ctx: &mut Self::Ctx, id_gen: &mut NextId) -> Self::Lower;
    /// Name used for debugging
    fn debug_name() -> &'static str;
}

#[allow(dead_code)]
pub(crate) struct LowerFontCtx<'a> {
    pub text_streams: LowerBox<'a, Ascii85Stream<'a>>,
    pub encodings: LowerBox<'a, Encoding<'a>>,
}

fn lower_font<'a>(
    font: &'a Font<'a>,
    ctx: &mut LowerFontCtx<'a>,
    id_gen: &mut NextId,
) -> low::Font<'a> {
    match font {
        Font::Type3(font) => {
            let char_procs = font
                .char_procs
                .iter()
                .map(|(key, proc)| {
                    let re = ctx.text_streams.put(proc, id_gen);
                    (key.clone(), re)
                })
                .collect();
            let to_unicode = font
                .to_unicode
                .as_ref()
                .map(|res| ctx.text_streams.map_ref(res, id_gen));
            low::Font::Type3(low::Type3Font {
                name: font.name,
                font_bbox: font.font_bbox,
                font_descriptor: font.font_descriptor.clone(),
                font_matrix: font.font_matrix,
                first_char: font.first_char,
                last_char: font.last_char,
                encoding: low::Resource::Immediate(font.encoding.clone()),
                char_procs,
                widths: &font.widths,
                to_unicode,
            })
        }
    }
}

impl<'a> Lowerable<'a> for Font<'a> {
    type Lower = low::Font<'a>;
    type Ctx = LowerFontCtx<'a>;

    fn lower(&'a self, ctx: &mut Self::Ctx, id_gen: &mut NextId) -> Self::Lower {
        lower_font(self, ctx, id_gen)
    }

    fn debug_name() -> &'static str {
        "Font"
    }
}

impl<'a> Lowerable<'a> for XObject {
    type Lower = low::XObject<'a>;
    type Ctx = ();

    fn lower(&'a self, _ctx: &mut Self::Ctx, _id_gen: &mut NextId) -> Self::Lower {
        match self {
            Self::Image(i) => low::XObject::Image(low::Ascii85Stream {
                data: Cow::Borrowed(&i.data),
                meta: StreamMetadata::Image(i.meta),
            }),
        }
    }

    fn debug_name() -> &'static str {
        "XObject"
    }
}

impl<'a> Lowerable<'a> for Ascii85Stream<'a> {
    type Lower = low::Ascii85Stream<'a>;
    type Ctx = ();

    fn lower(&self, _ctx: &mut Self::Ctx, _id_gen: &mut NextId) -> Self::Lower {
        low::Ascii85Stream {
            data: self.data.clone(),
            meta: self.meta,
        }
    }

    fn debug_name() -> &'static str {
        "CharProc"
    }
}

impl<'a> Lowerable<'a> for Encoding<'a> {
    type Lower = Encoding<'a>;
    type Ctx = ();

    fn lower(&self, _ctx: &mut Self::Ctx, _id_gen: &mut NextId) -> Self::Lower {
        self.clone()
    }

    fn debug_name() -> &'static str {
        "CharProc"
    }
}

pub(crate) struct LowerBox<'a, T> {
    pub store: HashMap<usize, (ObjRef, &'a T)>,
    res: &'a [T],
    next: usize,
}

impl<'a, T> LowerBox<'a, T> {
    fn new(res: &'a [T]) -> Self {
        LowerBox {
            store: HashMap::new(),
            res,
            next: res.len(),
        }
    }
}

pub(crate) fn lower_dict<'a, T: Lowerable<'a>>(
    dict: &'a DictResource<T>,
    inner: &mut LowerBox<'a, T>,
    ctx: &mut T::Ctx,
    id_gen: &mut NextId,
) -> low::DictResource<T::Lower> {
    dict.iter()
        .map(|(key, res)| (key.clone(), inner.map(res, ctx, id_gen)))
        .collect()
}

impl<'a, T: Lowerable<'a>> LowerBox<'a, DictResource<T>> {
    pub fn map_dict(
        &mut self,
        res: &'a ResDictRes<T>,
        inner: &mut LowerBox<'a, T>,
        ctx: &mut T::Ctx,
        id_gen: &mut NextId,
    ) -> low::ResDictRes<T::Lower> {
        match res {
            Resource::Global(GlobalResource { index, .. }) => {
                if let Some((r, _)) = self.store.get(index) {
                    low::Resource::Ref(*r)
                } else if let Some(font_dict) = self.res.get(*index) {
                    let id = id_gen.next();
                    let r = make_ref(id);
                    self.store.insert(*index, (r, font_dict));
                    low::Resource::Ref(r)
                } else {
                    panic!("Couldn't find {} Dict #{}", T::debug_name(), index);
                }
            }
            Resource::Immediate(fonts) => {
                let dict = lower_dict(fonts.as_ref(), inner, ctx, id_gen);
                low::Resource::Immediate(dict)
            }
        }
    }
}

impl<'a, T: Lowerable<'a>> LowerBox<'a, T> {
    /// Put a new object in the lower box
    fn put(&mut self, val: &'a T, id_gen: &mut NextId) -> ObjRef {
        let index = self.next;
        let r = self.val_ref(val, id_gen, index);
        self.next += 1;
        r
    }

    /// INTERNAL: assign a new ID, put to store
    fn val_ref(&mut self, val: &'a T, id_gen: &mut NextId, index: usize) -> ObjRef {
        let r = make_ref(id_gen.next());
        self.store.insert(index, (r, val));
        r
    }

    /// Lower the resource into an object ref
    ///
    /// Use this for objects that have to be indirect, e.g. streams
    fn map_ref(&mut self, res: &'a Resource<T>, id_gen: &mut NextId) -> ObjRef {
        match res {
            Resource::Global(global) => self.map_global_ref(id_gen, global),
            Resource::Immediate(content) => self.put(content, id_gen),
        }
    }

    fn map(
        &mut self,
        res: &'a Resource<T>,
        ctx: &mut T::Ctx,
        id_gen: &mut NextId,
    ) -> low::Resource<T::Lower> {
        match res {
            Resource::Global(global) => low::Resource::Ref(self.map_global_ref(id_gen, global)),
            Resource::Immediate(content) => low::Resource::Immediate(content.lower(ctx, id_gen)),
        }
    }

    fn map_global_ref(&mut self, id_gen: &mut NextId, global: &GlobalResource<T>) -> ObjRef {
        if let Some((r, _)) = self.store.get(&global.index) {
            *r
        } else if let Some(val) = self.res.get(global.index) {
            self.val_ref(val, id_gen, global.index)
        } else {
            panic!("Couldn't find {} #{}", T::debug_name(), global.index);
        }
    }
}

pub(crate) struct Lowering<'a> {
    pub id_gen: NextId,
    pub x_objects: LowerBox<'a, XObject>,
    pub x_object_dicts: LowerBox<'a, DictResource<XObject>>,
    pub fonts: LowerBox<'a, Font<'a>>,
    pub font_dicts: LowerBox<'a, DictResource<Font<'a>>>,
    pub font_ctx: LowerFontCtx<'a>,
}

impl<'a> Lowering<'a> {
    pub fn new(doc: &'a Handle) -> Self {
        Lowering {
            id_gen: NextId::new(1),
            x_objects: LowerBox::new(&doc.res.x_objects),
            x_object_dicts: LowerBox::new(&doc.res.x_object_dicts),
            fonts: LowerBox::new(&doc.res.fonts),
            font_dicts: LowerBox::new(&doc.res.font_dicts),
            font_ctx: LowerFontCtx {
                text_streams: LowerBox::new(&doc.res.char_procs),
                encodings: LowerBox::new(&doc.res.encodings),
            },
        }
    }
}
