//! # General utilities

mod bit_iter;
mod bit_writer;
mod buf;
mod bytes;
pub mod data;
mod file_format;
mod four_cc;
mod parsers;
mod vfs;

pub use bit_iter::{BitIter, ByteBits};
pub(crate) use bit_writer::BitWriter;
pub use buf::Buf;
pub use bytes::{Bytes16, Bytes32};
pub use file_format::{
    FileFormatKind, FileFormatKindV1, Signum1Format, Signum3Format, SignumFormat,
};
pub use four_cc::FourCC;
pub use parsers::V3Chunk;
#[allow(unused_imports)]
pub(crate) use parsers::{map_bstr, map_buf};
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
