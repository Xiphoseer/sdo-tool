//! # CCITT Group 4 2D-encoding
//! 
//! Spec: ITU-T Recommendation T.6 (11/88) <https://www.itu.int/rec/T-REC-T.6-198811-I/en>

use crate::bits::{BitIter, FillOrder};

mod decode;
mod decode_iter;
mod encode;

pub use decode::Decoder as G4Decoder;
use decode_iter::FaxDecode;
pub use decode_iter::FaxImage;
pub use encode::Encoder as G4Encoder;

/// Options for fax decoding
#[derive(Default)]
#[non_exhaustive]
pub struct FaxOptions {
    /// The width of the image
    pub width: usize,
    /// The order of bits in a byte
    pub fill_order: FillOrder,
    /// Print to console after decoding
    pub debug: bool,
}

/// An error when parsing a CCITT encoded bi-level image
#[non_exhaustive]
#[derive(Debug)]
pub enum FaxError {}
type FaxResult<T> = Result<T, FaxError>;

/// Decode a bitmap and print it to the console
///
/// **Note**: This does not use [`G4Decoder`]!
pub fn fax_decode(glyph_data: &[u8], options: FaxOptions) -> FaxResult<FaxImage> {
    let mut bit_iter = BitIter::new(glyph_data);
    bit_iter.set_fill_order(options.fill_order);
    let mut fax_decode = FaxDecode::new(options.width);
    fax_decode.set_debug(options.debug);
    fax_decode.decode(&mut bit_iter)
}
