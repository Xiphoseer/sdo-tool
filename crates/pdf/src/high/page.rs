use crate::common::{ProcSet, Rectangle};

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
