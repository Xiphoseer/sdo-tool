//! # Raster/Bitmap image processing

use std::{cmp::Ordering, fmt};

use crate::{
    chsets::{editor::EChar, printer::PSetChar},
    docs::hcim::ImageArea,
    images::imc::MonochromeScreen,
    util::bit_iter::BitIter,
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

struct VScaler<'a> {
    w: usize,
    h: usize,
    sel_h: usize,
    sel_w: usize,
    image: &'a Page,
    pixel_h_len: usize,
    pixel_v_len: usize,
    vpixel_count: usize,
    last_vcount: usize,
    iubpl: usize,
    ibyte_index: usize,
    skip_bits: u16,
    ivpixel_rem: usize,
    vpxl: bool,
}

impl<'a> VScaler<'a> {
    fn new(image: &'a Page, w: usize, h: usize, sel: ImageArea) -> Self {
        let iubpl = image.bytes_per_line as usize;
        let pixel_v_len = (h as usize) / (sel.h as usize);
        let ivpixel_rem = 0;
        let ibyte_index = (sel.y as usize) * iubpl + (sel.x as usize) / 8;
        let skip_bits = sel.x % 8;
        let vpxl = false;
        Self {
            sel_h: sel.h as usize,
            sel_w: sel.w as usize,
            w,
            h,
            image,
            pixel_h_len: (w as usize) / (sel.w as usize),
            pixel_v_len,
            vpixel_count: 0,
            last_vcount: 0,
            iubpl,
            ibyte_index,
            skip_bits,
            ivpixel_rem,
            vpxl,
        }
    }

    fn next_line<'b>(&'b mut self) -> HScaler<'a, 'b> {
        let mut ibit_iter = BitIter::new(&self.image.buffer[self.ibyte_index..]);
        let hpixel_count = 0;
        let last_hcount = 0;
        let hpxl = self.vpxl;
        let ipixel_rem = 0;
        for _ in 0..self.skip_bits {
            let _ = ibit_iter.next();
        }
        let icurr = ibit_iter.next().unwrap_or(true);
        HScaler {
            vscaler: self,
            ibit_iter,
            hpixel_count,
            last_hcount,
            hpxl,
            ipixel_rem,
            icurr,
        }
    }
}

struct HScaler<'a, 'b> {
    vscaler: &'b mut VScaler<'a>,
    ibit_iter: BitIter<'a>,
    hpixel_count: usize,
    last_hcount: usize,
    hpxl: bool,
    ipixel_rem: usize,
    icurr: bool,
}

impl<'a, 'b> HScaler<'a, 'b> {
    fn next(&mut self) -> bool {
        if self.vscaler.pixel_h_len == 0 {
            while self.last_hcount < self.hpixel_count * self.vscaler.sel_w / self.vscaler.w {
                if self.ipixel_rem == 7 {
                    self.hpxl = !self.hpxl;
                    //self.icurr = self.hpxl;
                    self.ipixel_rem = 0;
                } else {
                    self.ipixel_rem += 1;
                }
                self.icurr = self.ibit_iter.next().unwrap();
                self.last_hcount += 1;
            }
        } else {
            let hcount = self.hpixel_count * self.vscaler.sel_w / self.vscaler.w;
            if self.last_hcount < hcount {
                if self.ipixel_rem == 7 {
                    self.hpxl = !self.hpxl;
                    //self.icurr = self.hpxl;
                    self.ipixel_rem = 0;
                } else {
                    self.ipixel_rem += 1;
                }
                self.icurr = self.ibit_iter.next().unwrap();
                self.last_hcount += 1;
            }
        }
        self.hpixel_count += 1;
        self.icurr
    }

    fn end(self) {
        let vs = self.vscaler;
        /*println!(
            "lhcount: {:4}, hpixel: {:4}, sel_w: {:4}, w: {:4}",
            self.last_hcount, self.hpixel_count, vs.sel_w, vs.w
        );
        println!(
            "lvcount: {:4}, vpixel: {:4}, sel_h: {:4}, h: {:4}",
            vs.last_vcount, vs.vpixel_count, vs.sel_h, vs.h
        );*/

        //vs.ivpixel_rem = vs.pixel_v_len;
        if vs.pixel_v_len == 0 {
            while vs.last_vcount < vs.vpixel_count * vs.sel_h / vs.h {
                vs.last_vcount += 1;
                vs.ibyte_index += vs.iubpl;
                if vs.ivpixel_rem == 7 {
                    vs.ivpixel_rem = 0;
                    vs.vpxl = !vs.vpxl;
                } else {
                    vs.ivpixel_rem += 1;
                }
            }
        } else {
            let vcount = vs.vpixel_count * vs.sel_h / vs.h;
            if vs.last_vcount < vcount {
                vs.ibyte_index += vs.iubpl;
                if vs.ivpixel_rem == 7 {
                    vs.ivpixel_rem = 0;
                    vs.vpxl = !vs.vpxl;
                } else {
                    vs.ivpixel_rem += 1;
                }
                vs.last_vcount += 1;
            }
        }

        vs.vpixel_count += 1;
    }
}

impl Page {
    /// Turn a (fixed-size) screen into a (variable-sized) page
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

impl Page {
    /// Create a new page with the given dimensions
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

    /// Draw a single printer char on the page
    pub fn draw_printer_char(&mut self, x: u32, y: u32, ch: &PSetChar) -> Result<(), DrawPrintErr> {
        let width = usize::from(ch.width);
        let height = usize::from(ch.height);
        let top = usize::from(ch.top);

        if width == 0 || height == 0 {
            return Ok(());
        }

        let ux = x as usize;
        let uy = y as usize;

        if ux + width + 1 > (self.width as usize) {
            return Err(DrawPrintErr::OutOfBounds);
        }

        if uy + height + top + 1 > (self.height as usize) {
            return Err(DrawPrintErr::OutOfBounds);
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

    /// Draw a single editor char on the page
    pub fn draw_echar(&mut self, x: u16, y: u16, ch: &EChar) -> Result<(), DrawPrintErr> {
        if u32::from(x + u16::from(ch.width)) + 2 >= self.width {
            return Err(DrawPrintErr::OutOfBounds);
        }

        if u32::from(y + u16::from(ch.height + ch.top)) + 2 >= self.height {
            return Err(DrawPrintErr::OutOfBounds);
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

    /// Draw an image on the page
    pub fn draw_image(&mut self, px: u32, py: u32, w: u32, h: u32, image: &Self, sel: ImageArea) {
        let ubpl = self.bytes_per_line as usize;
        let mut byte_index = (py as usize) * ubpl + (px as usize) / 8;
        let bit_offset = px % 8;
        let first_bit_rem = 8 - bit_offset;

        let mut vscaler = VScaler::new(image, w as usize, h as usize, sel);

        match w.cmp(&first_bit_rem) {
            Ordering::Less => {
                let b: u8 = match w {
                    0 => return,
                    1 => 0b10000000,
                    2 => 0b11000000,
                    3 => 0b11100000,
                    4 => 0b11110000,
                    5 => 0b11111000,
                    6 => 0b11111100,
                    7 => 0b11111110,
                    _ => unreachable!(),
                } >> bit_offset;
                for _ in 0..h {
                    self.buffer[byte_index] |= b;
                    byte_index += ubpl;
                }
            }
            Ordering::Equal => {
                let b = 0xff >> bit_offset;
                for _ in 0..h {
                    self.buffer[byte_index] |= b;
                    byte_index += ubpl;
                }
            }
            Ordering::Greater => {
                let do_first = bit_offset != 0;
                let bit_rem = if do_first { w - first_bit_rem } else { w };
                let count = bit_rem / 8;
                let bit_last = bit_rem % 8;
                let do_last = bit_last != 0;

                let blen = self.buffer.len();

                for _ in 0..h {
                    if byte_index >= blen {
                        println!("Image box out of bounds");
                        return;
                    }
                    let mut row_index = byte_index;
                    let mut hscaler = vscaler.next_line();
                    if do_first {
                        let mut r = bit_offset;
                        let mut first = 0x00;
                        while r > 0 {
                            first <<= 1;
                            if hscaler.next() {
                                first |= 0x01;
                            }
                            r -= 1;
                        }
                        self.buffer[row_index] |= first;
                        row_index += 1;
                    }
                    for _ in 0..count {
                        if row_index >= blen {
                            println!("Image box out of bounds");
                            return;
                        }
                        let mut b = 0u8;
                        for _ in 0..8 {
                            b <<= 1;
                            if hscaler.next() {
                                b |= 0x01;
                            }
                        }
                        self.buffer[row_index] |= b;
                        row_index += 1;
                    }
                    if do_last {
                        if row_index >= blen {
                            println!("Image box out of bounds");
                            return;
                        }
                        let mut r = bit_last;
                        let mut last = 0x00;
                        while r > 0 {
                            last >>= 1;
                            if hscaler.next() {
                                last |= 0x80;
                            }
                            r -= 1;
                        }
                        self.buffer[row_index] |= last;
                    }
                    byte_index += ubpl;
                    hscaler.end();
                }
            }
        }
    }

    /// Turn the page into a `GrayImage` from the `image` crate
    pub fn to_image(&self) -> GrayImage {
        let mut buffer = Vec::with_capacity(self.buffer.len() * 8);
        for byte in self.buffer.iter().map(|b| *b as usize) {
            buffer.extend_from_slice(&BIT_PROJECTION[byte]);
        }
        GrayImage::from_vec(self.bytes_per_line * 8, self.height, buffer).unwrap()
    }
}
