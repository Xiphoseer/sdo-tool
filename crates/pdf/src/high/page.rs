use crate::{
    common::{ObjRef, ProcSet, Rectangle},
    low,
    lowering::{DebugName, LowerPagesCtx},
    util::NextId,
};

use super::{Font, ResDictRes, Resource, XObject};

/// A single page
pub struct Page<'a> {
    /// The dimensions of the page
    pub media_box: Rectangle<i32>,
    /// The resource used within the page
    pub resources: Resources<'a>,
    /// The content stream of the page
    pub contents: Vec<u8>,
}

/// The resources of a page
pub struct Resources<'a> {
    /// A dict of font resources
    pub fonts: ResDictRes<Font<'a>>,
    /// A dict of embedded object resources
    pub x_objects: ResDictRes<XObject>,
    /// A set of valid procedures
    pub proc_sets: Vec<ProcSet>,
}

impl Default for Resources<'_> {
    fn default() -> Self {
        Resources {
            fonts: Resource::Immediate(Box::default()),
            x_objects: Resource::Immediate(Box::default()),
            proc_sets: vec![ProcSet::PDF, ProcSet::Text],
        }
    }
}

impl DebugName for Page<'_> {
    fn debug_name() -> &'static str {
        "Page"
    }
}

pub(crate) fn lower_page<'a>(
    page: &'a Page<'a>,
    ctx: &mut LowerPagesCtx<'a>,
    id_gen: &mut NextId,
    contents_ref: ObjRef,
) -> low::Page<'a> {
    low::Page {
        parent: ctx.pages_ref,
        resources: low::Resources {
            font: ctx.font_dicts.map_dict(
                &page.resources.fonts,
                &mut ctx.fonts,
                &mut ctx.font_ctx,
                id_gen,
            ),
            x_object: ctx.x_object_dicts.map_stream_dict(
                &page.resources.x_objects,
                &mut ctx.x_objects,
                id_gen,
            ),
            proc_set: &page.resources.proc_sets,
        },
        contents: contents_ref,
        media_box: Some(page.media_box),
    }
}
