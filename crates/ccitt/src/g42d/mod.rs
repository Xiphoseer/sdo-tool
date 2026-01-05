//! CCITT Group 4 2D-encoding

use super::bits::BitIter;

mod decode;
mod decode_iter;
mod encode;

pub use decode::{Decoder, Store, ColorLine};
use decode_iter::FaxDecode;
pub use encode::Encoder;

/// Decode a bitmap and print it to the console
///
/// **Note**: This does not use [`Decoder`]!
pub fn fax_decode(glyph_data: &[u8], width: usize) {
    let mut bit_iter = BitIter::new(glyph_data);
    let mut fax_decode = FaxDecode::new(width);

    fax_decode.decode(&mut bit_iter);
    fax_decode.print();
}
