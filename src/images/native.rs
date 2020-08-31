use super::BitIter;
use image::GrayImage;
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct DecodeError(usize);
impl Error for DecodeError {}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Monochrome image is {} bytes, expected 32000", self.0)
    }
}

pub fn decode_monochrome(src: &[u8]) -> Result<GrayImage, DecodeError> {
    let bit_iter = BitIter::new(src);
    let buffer: Vec<u8> = bit_iter.map(|b| if b { 0 } else { 255 }).collect();
    GrayImage::from_vec(640, 400, buffer).ok_or_else(|| DecodeError(src.len()))
}
