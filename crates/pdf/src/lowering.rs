//! Helpers to turn *high* types into *low* types

use pdf::object::PlainRef;

use crate::{
    common::Encoding, high::CharProc, high::Destination, high::DictResource, high::Font,
    high::Handle, high::OutlineItem, high::ResDictRes, high::Resource, high::XObject, low,
    util::NextID,
};

/// Make a PlainRef for an original document (generation 0)
pub fn make_ref(id: u64) -> PlainRef {
    PlainRef { id, gen: 0 }
}

fn lower_dest(pages: &[PlainRef], dest: Destination) -> low::Action {
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
    acc: &mut Vec<(PlainRef, low::OutlineItem)>,
    pages: &[PlainRef],
    items: &[OutlineItem],
    parent: PlainRef,
    id_gen: &mut NextID,
) -> Option<(PlainRef, PlainRef)> {
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

    fn lower(&'a self, ctx: &mut Self::Ctx, id_gen: &mut NextID) -> Self::Lower;
    fn name() -> &'static str;
}

type LowerFontCtx<'a> = (LowerBox<'a, CharProc<'a>>, LowerBox<'a, Encoding<'a>>);

fn lower_font<'a>(
    font: &'a Font<'a>,
    (a, _b): &mut LowerFontCtx<'a>,
    id_gen: &mut NextID,
) -> low::Font<'a> {
    match font {
        Font::Type3(font) => {
            let char_procs = font
                .char_procs
                .iter()
                .map(|(key, proc)| {
                    let re = a.put(proc, id_gen);
                    (key.clone(), re)
                })
                .collect();
            low::Font::Type3(low::Type3Font {
                name: font.name,
                font_bbox: font.font_bbox,
                font_matrix: font.font_matrix,
                first_char: font.first_char,
                last_char: font.last_char,
                encoding: low::Resource::Immediate(font.encoding.clone()),
                char_procs,
                widths: &font.widths,
            })
        }
    }
}

impl<'a> Lowerable<'a> for Font<'a> {
    type Lower = low::Font<'a>;
    type Ctx = LowerFontCtx<'a>;

    fn lower(&'a self, ctx: &mut Self::Ctx, id_gen: &mut NextID) -> Self::Lower {
        lower_font(self, ctx, id_gen)
    }

    fn name() -> &'static str {
        "Font"
    }
}

impl<'a> Lowerable<'a> for XObject {
    type Lower = low::XObject;
    type Ctx = ();

    fn lower(&'a self, _ctx: &mut Self::Ctx, _id_gen: &mut NextID) -> Self::Lower {
        todo!()
    }

    fn name() -> &'static str {
        "XObject"
    }
}

impl<'a> Lowerable<'a> for CharProc<'a> {
    type Lower = low::CharProc<'a>;
    type Ctx = ();

    fn lower(&self, _ctx: &mut Self::Ctx, _id_gen: &mut NextID) -> Self::Lower {
        low::CharProc(self.0.clone())
    }

    fn name() -> &'static str {
        "CharProc"
    }
}

impl<'a> Lowerable<'a> for Encoding<'a> {
    type Lower = Encoding<'a>;
    type Ctx = ();

    fn lower(&self, _ctx: &mut Self::Ctx, _id_gen: &mut NextID) -> Self::Lower {
        self.clone()
    }

    fn name() -> &'static str {
        "CharProc"
    }
}

pub(crate) struct LowerBox<'a, T> {
    pub store: Vec<(PlainRef, &'a T)>,
    res: &'a [T],
}

impl<'a, T> LowerBox<'a, T> {
    fn new(res: &'a [T]) -> Self {
        LowerBox { store: vec![], res }
    }
}

pub(crate) fn lower_dict<'a, T: Lowerable<'a>>(
    dict: &'a DictResource<T>,
    inner: &mut LowerBox<'a, T>,
    ctx: &mut T::Ctx,
    id_gen: &mut NextID,
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
        id_gen: &mut NextID,
    ) -> low::ResDictRes<T::Lower> {
        match res {
            Resource::Global { index } => {
                if let Some((r, _)) = self.store.get(*index) {
                    low::Resource::Ref(*r)
                } else if let Some(font_dict) = self.res.get(*index) {
                    let id = id_gen.next();
                    let r = make_ref(id);
                    self.store.push((r, font_dict));
                    low::Resource::Ref(r)
                } else {
                    panic!("Couldn't find {} Dict #{}", T::name(), index);
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
    fn put(&mut self, val: &'a T, id_gen: &mut NextID) -> PlainRef {
        let id = id_gen.next();
        let r = make_ref(id);
        self.store.push((r, val));
        r
    }

    fn map(
        &mut self,
        res: &'a Resource<T>,
        ctx: &mut T::Ctx,
        id_gen: &mut NextID,
    ) -> low::Resource<T::Lower> {
        match res {
            Resource::Global { index } => {
                if let Some((r, _)) = self.store.get(*index) {
                    low::Resource::Ref(*r)
                } else if let Some(val) = self.res.get(*index) {
                    let id = id_gen.next();
                    let r = make_ref(id);
                    self.store.push((r, val));
                    low::Resource::Ref(r)
                } else {
                    panic!("Couldn't find {} #{}", T::name(), index);
                }
            }
            Resource::Immediate(content) => {
                let content_low = content.lower(ctx, id_gen);
                low::Resource::Immediate(content_low)
            }
        }
    }
}

pub(crate) struct Lowering<'a> {
    pub id_gen: NextID,
    pub x_objects: LowerBox<'a, XObject>,
    pub x_object_dicts: LowerBox<'a, DictResource<XObject>>,
    pub fonts: LowerBox<'a, Font<'a>>,
    pub font_dicts: LowerBox<'a, DictResource<Font<'a>>>,
    pub font_ctx: LowerFontCtx<'a>,
}

impl<'a> Lowering<'a> {
    pub fn new(doc: &'a Handle) -> Self {
        Lowering {
            id_gen: NextID::new(1),
            x_objects: LowerBox::new(&doc.res.x_objects),
            x_object_dicts: LowerBox::new(&doc.res.x_object_dicts),
            fonts: LowerBox::new(&doc.res.fonts),
            font_dicts: LowerBox::new(&doc.res.font_dicts),
            font_ctx: (
                LowerBox::new(&doc.res.char_procs),
                LowerBox::new(&doc.res.encodings),
            ),
        }
    }
}
