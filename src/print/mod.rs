use crate::{font::eset::EChar, images::BitIter, util::BIT_STRING};
use image::GrayImage;

/// A virtual page that works just like the atari monochrome screen
///
/// The width and height are in pixels. The width works best if it is a
/// multiple of 8. Every u8 in the buffer is represents 8 sequential
/// pixels in a row where 0 is white (no ink) and 1 is black (ink).
pub struct Page {
    bytes_per_line: u32,
    width: u32,
    height: u32,
    buffer: Vec<u8>,
}

fn print(bytes_per_line: u32, width: u32, buffer: &[u8]) {
    let _final_bits = width % 8;
    let border = || {
        print!("+");
        for _ in 0..bytes_per_line {
            print!("--------");
        }
        println!("+");
    };

    border();
    for line in buffer.chunks_exact(bytes_per_line as usize) {
        print!("|");
        for byte in line.iter().copied() {
            print!("{}", &BIT_STRING[byte as usize]);
        }
        println!("|");
    }
    border();
}

impl Page {
    pub fn new(width: u32, height: u32) -> Self {
        let bytes_per_line = (width - 1) / 8 + 1;
        Page {
            bytes_per_line,
            width,
            height,
            buffer: vec![0; (bytes_per_line as usize) * (height as usize)],
        }
    }

    /// This function prints an images to the console.
    ///
    /// Use this for small images only
    pub fn print(&self) {
        print(self.bytes_per_line, self.width, &self.buffer);
    }

    pub fn draw_char(&mut self, x: u16, y: u16, ch: &EChar) -> Result<(), ()> {
        if u32::from(x + u16::from(ch.width)) + 2 >= self.width {
            return Err(());
        }

        if u32::from(y + u16::from(ch.height + ch.top)) + 2 >= self.height {
            return Err(());
        }

        let y_byte = u32::from(y + ch.top as u16) * self.bytes_per_line;
        let x_byte = u32::from(x) / 8;
        let x_bit = x % 8;

        let mut byte_index: usize = (y_byte + x_byte) as usize;

        if x_bit == 0 {
            for x in 0..(ch.height as usize) {
                self.buffer[byte_index] |= ch.buf[x * 2];
                self.buffer[byte_index + 1] |= ch.buf[x * 2 + 1];
                byte_index += self.bytes_per_line as usize;
            }
        } else {
            for x in 0..(ch.height as usize) {
                let full = u32::from_be_bytes([0, 0, ch.buf[x * 2], ch.buf[x * 2 + 1]]);
                let shifted = full << (8 - x_bit);
                let [_, byte0, byte1, byte2] = shifted.to_be_bytes();

                self.buffer[byte_index] |= byte0;
                self.buffer[byte_index + 1] |= byte1;
                self.buffer[byte_index + 2] |= byte2;

                byte_index += self.bytes_per_line as usize;
            }
        }

        Ok(())
    }

    pub fn from_screen(buffer: Vec<u8>) -> Result<Self, Vec<u8>> {
        if buffer.len() == 32000 {
            Ok(Page {
                bytes_per_line: 80,
                width: 640,
                height: 400,
                buffer,
            })
        } else {
            Err(buffer)
        }
    }

    pub fn to_image(&self) -> GrayImage {
        let bit_iter = BitIter::new(&self.buffer);
        let buffer: Vec<u8> = bit_iter.map(|b| if b { 0 } else { 255 }).collect();
        GrayImage::from_vec(self.bytes_per_line * 8, self.height, buffer).unwrap()
    }
}
