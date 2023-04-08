//! # Raster/Bitmap image processing

use std::fmt;

mod page;
mod scalers;
mod util;

pub use page::Page;

#[derive(Debug)]
/// Drawing Error
pub enum DrawPrintErr {
    /// The specified position was out of bounds
    OutOfBounds,
}

impl std::error::Error for DrawPrintErr {}
impl fmt::Display for DrawPrintErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBounds => write!(f, "Failed to draw character: out of bounds"),
        }
    }
}
