use crate::{
    font::{eset::EChar, ps24::PSetChar},
    images::imc::MonochromeScreen,
    util::data::{BIT_PROJECTION, BIT_STRING},
};
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

impl Page {
    pub fn from_screen(screen: MonochromeScreen) -> Self {
        Page {
            bytes_per_line: 80,
            width: 640,
            height: 400,
            buffer: screen.into_inner(),
        }
    }
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

    pub fn draw_char_p24(&mut self, x: u32, y: u32, ch: &PSetChar) -> Result<(), ()> {
        let width = usize::from(ch.width);
        let height = usize::from(ch.height);
        let top = usize::from(ch.top);

        if width == 0 || height == 0 {
            return Ok(());
        }

        let ux = x as usize;
        let uy = y as usize;

        if ux + width + 1 > (self.width as usize) {
            return Err(());
        }

        if uy + height + top + 1 > (self.height as usize) {
            return Err(());
        }

        let ubpl = self.bytes_per_line as usize;
        let x_bit = x % 8;
        let mut base_index = ((uy + top) * ubpl + ux / 8) as usize;
        if x_bit == 0 {
            for row in ch.bitmap.chunks_exact(width as usize) {
                let mut row_index = base_index;
                for byte in row {
                    self.buffer[row_index] |= *byte;
                    row_index += 1;
                }
                base_index += ubpl;
            }
        } else {
            let x_shift = 8 - x_bit;
            for row in ch.bitmap.chunks_exact(width as usize) {
                let mut row_index = base_index;
                let mut next = 0x00;
                for byte in row {
                    self.buffer[row_index] |= (byte >> x_bit) | next;
                    next = byte << x_shift;
                    row_index += 1;
                }
                self.buffer[row_index] |= next;
                base_index += ubpl;
            }
        }

        Ok(())
    }

    pub fn draw_echar(&mut self, x: u16, y: u16, ch: &EChar) -> Result<(), ()> {
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

    pub fn to_image(&self) -> GrayImage {
        let mut buffer = Vec::with_capacity(self.buffer.len() * 8);
        for byte in self.buffer.iter().map(|b| *b as usize) {
            buffer.extend_from_slice(&BIT_PROJECTION[byte]);
        }
        GrayImage::from_vec(self.bytes_per_line * 8, self.height, buffer).unwrap()
    }
}
