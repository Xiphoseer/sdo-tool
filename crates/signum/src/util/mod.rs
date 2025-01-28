//! # General utilities

mod bit_iter;
mod bit_writer;
mod buf;
mod bytes;
pub mod data;
mod four_cc;
mod vfs;

pub use bit_iter::BitIter;
pub(crate) use bit_writer::BitWriter;
pub use buf::Buf;
pub use bytes::{Bytes16, Bytes32};
pub use four_cc::FourCC;
pub use vfs::{AsyncIterator, LocalFS, VFS};

/// A 16 bit position
pub struct Pos {
    /// horizontal
    pub x: u16,
    /// vertical
    pub y: u16,
}

impl Pos {
    /// Create a new point
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}
