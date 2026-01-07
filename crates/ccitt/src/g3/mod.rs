//! # CCITT Group 3 1D-encoding
//!
//! TODO: 2-D encoding
//!
//! Spec: ITU-T Recommendation T.4 (07/03) <https://www.itu.int/rec/T-REC-T.4-200307-I/en>
use crate::{bits::BitIter, FaxImage, FaxResult};

mod decode_iter;

/// # Group 3 (T.4) Decoder
pub struct G3Decoder {
    width: usize,
}

impl G3Decoder {
    /// Create a new instance
    pub fn new(width: usize) -> Self {
        Self { width }
    }

    /// Decode a Group 3 image
    pub fn decode(&mut self, bit_iter: &mut BitIter<'_>) -> FaxResult<FaxImage> {
        let mut image = Vec::new();
        loop {
            let line = decode_iter::decode_1d_line(bit_iter, self.width);
            if line.is_empty() {
                break;
            } else {
                image.extend_from_slice(&line);
            }
        }
        Ok(FaxImage {
            width: self.width,
            complete: image,
        })
    }
}
