use crate::common::PdfString;

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
