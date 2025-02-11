use std::{cmp::Ordering, collections::VecDeque, slice::SliceIndex};

#[cfg(feature = "image")]
use image::{GrayAlphaImage, GrayImage};

use crate::{
    chsets::{editor::EChar, printer::PSetChar},
    docs::hcim::ImageArea,
    images::imc::MonochromeScreen,
    util::data::BIT_PROJECTION,
    util::BitIter,
    util::BitWriter,
};

use super::{scalers::VScaler, trace::Dir, DrawPrintErr};

/// A virtual page that works just like the atari monochrome screen
///
/// The width and height are in pixels. The width works best if it is a
/// multiple of 8. Every u8 in the buffer is represents 8 sequential
/// pixels in a row where 0 is white (no ink) and 1 is black (ink).
#[derive(Clone)]
pub struct Page {
    bytes_per_line: u32,
    width: u32,
    height: u32,
    buffer: Vec<u8>,
}

impl From<MonochromeScreen> for Page {
    /// Turn a (fixed-size) screen into a (variable-sized) page
    fn from(screen: MonochromeScreen) -> Self {
        Page {
            bytes_per_line: 80,
            width: 640,
            height: 400,
            buffer: screen.into_inner(),
        }
    }
}

impl From<&'_ PSetChar<'_>> for Page {
    fn from(value: &PSetChar<'_>) -> Self {
        let bytes_per_line = value.width.into();
        Page {
            bytes_per_line,
            width: bytes_per_line * 8,
            height: value.height.into(),
            buffer: value.bitmap.to_vec(),
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

    /// The width in B/W pixels
    pub fn bit_width(&self) -> u32 {
        self.width
    }

    /// The height in B/W pixels
    pub fn bit_height(&self) -> u32 {
        self.height
    }

    /// Return the number of bytes per line
    pub fn bytes_per_line(&self) -> u32 {
        self.bytes_per_line
    }

    /// Get a bit iterator for the given byte range
    pub fn bits<R: SliceIndex<[u8], Output = [u8]>>(&self, range: R) -> BitIter {
        BitIter::new(&self.buffer[range])
    }

    /// Get the position of the "first" pixel with ink
    pub fn first_ink(&self) -> Option<(u32, u32)> {
        self.buffer.iter().position(|&x| x > 0).map(|index| {
            let xoff = self.buffer[index].leading_zeros();
            let y = index as u32 / self.bytes_per_line;
            let x = index as u32 % self.bytes_per_line;
            (x * 8 + xoff, y)
        })
    }

    /// Get the first outline
    pub fn first_outline(&self) -> Option<impl Iterator<Item = (u32, u32)> + '_> {
        self.first_ink().map(|p0| {
            let (x0, y0) = p0;
            let mut p = (x0, y0 + 1);
            let mut first = VecDeque::from([p0, p]);
            let mut dir = Dir::Down;
            std::iter::from_fn(move || {
                if let Some(x) = first.pop_front() {
                    return Some(x);
                }
                if p == p0 {
                    return None;
                }
                let (x, y) = p;
                dir = match dir {
                    Dir::Down => {
                        // left is blank, right is ink
                        let bl = x > 0 && self.ink_at(x - 1, y);
                        let br = self.ink_at(x, y);
                        match (bl, br) {
                            (true, _) => Dir::Left,
                            (false, true) => Dir::Down,
                            (false, false) => Dir::Right,
                        }
                    }
                    Dir::Up => {
                        // left is ink, right is blank
                        let tl = x > 0 && y > 0 && self.ink_at(x - 1, y - 1);
                        let tr = y > 0 && self.ink_at(x, y - 1);
                        match (tl, tr) {
                            (_, true) => Dir::Right,
                            (true, false) => Dir::Up,
                            (false, false) => Dir::Left,
                        }
                    }
                    Dir::Left => {
                        // top is blank, bottom is ink
                        let tl = x > 0 && y > 0 && self.ink_at(x - 1, y - 1);
                        let bl = x > 0 && self.ink_at(x - 1, y);
                        match (bl, tl) {
                            (_, true) => Dir::Up,
                            (true, false) => Dir::Left,
                            (false, false) => Dir::Down,
                        }
                    }
                    Dir::Right => {
                        // top is ink, bottom is blank
                        let tr = y > 0 && self.ink_at(x, y - 1);
                        let br = self.ink_at(x, y);
                        match (tr, br) {
                            (_, true) => Dir::Down,
                            (true, false) => Dir::Right,
                            (false, false) => Dir::Up,
                        }
                    }
                };
                p = match dir {
                    Dir::Down => (x, y + 1),
                    Dir::Up => (x, y - 1),
                    Dir::Left => (x - 1, y),
                    Dir::Right => (x + 1, y),
                };
                Some(p)
            })
        })
    }

    /// check whether there is ink at a given coordinate
    pub fn ink_at(&self, x: u32, y: u32) -> bool {
        if x >= self.width {
            return false;
        }
        let xb = x / 8;
        let shift = 7 - x % 8;
        let byte = (y * self.bytes_per_line + xb) as usize;
        if self.buffer.len() <= byte {
            return false;
        }
        ((self.buffer[byte] >> shift) & 1) > 0
    }

    /// Get an iterator of all vertices in the character
    pub fn vertices(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        const NONE: [bool; 4] = [false; 4];
        const ALL: [bool; 4] = [true; 4];
        self.points().filter(move |&(x, y)| {
            let a = x > 0 && y > 0 && self.ink_at(x - 1, y - 1);
            let b = y > 0 && self.ink_at(x, y - 1);
            let c = x > 0 && self.ink_at(x - 1, y);
            let d = self.ink_at(x, y);
            !matches!([a, b, c, d], NONE | ALL)
        })
    }

    /// Iterate over all points in the bitmap
    pub fn points(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        let (mut x, mut y): (u32, u32) = (0, 0);
        std::iter::from_fn(move || {
            let mut ret = None;
            if y <= self.height {
                if x <= self.width {
                    ret = Some((x, y));
                    x = (x + 1) % (self.width + 1);
                }
                if x == 0 {
                    y += 1;
                }
            }
            ret
        })
    }

    /// This function prints an images to the console.
    ///
    /// Use this for small images only
    pub fn print(&self) {
        super::util::print(self.bytes_per_line, self.width, &self.buffer);
    }

    /// Clear the page
    pub fn clear(&mut self) {
        self.buffer.fill(0);
    }

    /// Get a part of the image as bitmap (1-bit per pixel) image
    pub fn select(&self, area: ImageArea) -> Vec<u8> {
        let h = area.h as usize;
        let w = area.w as usize;
        let y = area.y as usize;
        let x = area.x as usize;
        let chunk_size = self.bytes_per_line as usize;
        let iter = self.buffer.chunks(chunk_size).skip(y).take(h);
        let mut out;
        let lskip = x / 8;
        let lmod = x % 8;

        if lmod > 0 {
            let mut bw = BitWriter::new();
            let lead = 8 - lmod;
            let end = w - lead;
            let cnt = end / 8;
            let rmod = end % 8;
            let apos = lskip + 1;
            let bpos = apos + cnt;
            for line in iter {
                bw.write_bits(line[lskip] as usize, lead);
                for &val in &line[apos..bpos] {
                    bw.write_bits(val as usize, 8);
                }
                if rmod > 0 {
                    bw.write_bits(line[bpos] as usize, rmod);
                }
                bw.flush();
            }
            out = bw.done();
        } else {
            out = Vec::with_capacity(w * h / 8 + 1);
            let rmod = w % 8;
            let len = if rmod > 0 { w / 8 + 1 } else { w / 8 };
            for line in iter {
                out.extend_from_slice(&line[lskip..lskip + len]);
            }
        }

        out
    }

    /// Get a part of the image as a grayscale (8-bit per pixel) image
    pub fn select_grayscale(&self, area: ImageArea) -> Vec<u8> {
        let h = area.h as usize;
        let w = area.w as usize;
        let y = area.y as usize;
        let x = area.x as usize;
        let chunk_size = self.bytes_per_line as usize;
        let iter = self.buffer.chunks(chunk_size).skip(y).take(h);
        let mut out = Vec::with_capacity(w * h);
        for line in iter {
            let mut lskip = x / 8;
            let lmod = x % 8;
            let mut lw = lmod + w;
            if lmod > 0 {
                let lbyte = line[lskip] as usize;
                let lsl = &BIT_PROJECTION[lbyte];
                if lmod + w <= 8 {
                    out.extend_from_slice(&lsl[lmod..lw]);
                    continue;
                } else {
                    lskip += 1;
                    lw -= 8;
                    out.extend_from_slice(&lsl[lmod..]);
                }
            }
            let mlen = lw / 8;
            let rmod = lw % 8;
            let rpos = lskip + mlen;
            for &byte in &line[lskip..rpos] {
                let mbyte = byte as usize;
                let sl = &BIT_PROJECTION[mbyte];
                out.extend_from_slice(sl);
            }
            if rmod > 0 {
                let rbyte = line[rpos] as usize;
                let rsl = &BIT_PROJECTION[rbyte];
                out.extend_from_slice(&rsl[..rmod]);
            }
        }
        out
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
        let mut base_index = (uy + top) * ubpl + ux / 8;
        if x_bit == 0 {
            for row in ch.bitmap.chunks_exact(width) {
                let mut row_index = base_index;
                for byte in row {
                    self.buffer[row_index] |= *byte;
                    row_index += 1;
                }
                base_index += ubpl;
            }
        } else {
            let x_shift = 8 - x_bit;
            for row in ch.bitmap.chunks_exact(width) {
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
        if u32::from(x + u16::from(ch.width)) > self.width {
            return Err(DrawPrintErr::OutOfBounds);
        }

        if u32::from(y + u16::from(ch.height + ch.top)) > self.height {
            return Err(DrawPrintErr::OutOfBounds);
        }

        let y_byte = u32::from(y + ch.top as u16) * self.bytes_per_line;
        let x_byte = u32::from(x) / 8;
        let x_bit = x % 8;

        // let wide = ch.width > 8;
        let cols_avail = self.bytes_per_line - x_byte;

        let mut byte_index: usize = (y_byte + x_byte) as usize;

        let second_byte = cols_avail > 1;
        if x_bit == 0 {
            for y in 0..(ch.height as usize) {
                self.buffer[byte_index] |= ch.buf[y * 2];
                if second_byte {
                    self.buffer[byte_index + 1] |= ch.buf[y * 2 + 1];
                }
                byte_index += self.bytes_per_line as usize;
            }
        } else {
            let third_byte = cols_avail > 2;
            for y in 0..(ch.height as usize) {
                let full = u32::from_be_bytes([0, 0, ch.buf[y * 2], ch.buf[y * 2 + 1]]);
                let shifted = full << (8 - x_bit);
                let [_, byte0, byte1, byte2] = shifted.to_be_bytes();

                self.buffer[byte_index] |= byte0;
                if second_byte {
                    self.buffer[byte_index + 1] |= byte1;
                }
                if third_byte {
                    self.buffer[byte_index + 2] |= byte2;
                }

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

    #[cfg(feature = "image")]
    #[cfg_attr(docsrs, doc(cfg(feature = "image")))]
    /// Turn the page into a `GrayImage` from the `image` crate
    pub fn to_image(&self) -> GrayImage {
        let mut buffer = Vec::with_capacity(self.buffer.len() * 8);
        for byte in self.buffer.iter().map(|b| *b as usize) {
            buffer.extend_from_slice(&BIT_PROJECTION[byte]);
        }
        GrayImage::from_vec(self.bytes_per_line * 8, self.height, buffer).unwrap()
    }

    #[cfg(feature = "image")]
    #[cfg_attr(docsrs, doc(cfg(feature = "image")))]
    /// Turn the page into a `GrayImage` from the `image` crate
    pub fn to_alpha_image(&self) -> GrayAlphaImage {
        let mut buffer = Vec::with_capacity(self.buffer.len() * 8);
        for byte in self.buffer.iter().copied() {
            let mut mask = 0b10000000;
            while mask > 0 {
                if byte & mask > 0 {
                    // bit set -> black
                    buffer.extend_from_slice(&[0x00, 0xFF]); // no color, full alpha
                } else {
                    buffer.extend_from_slice(&[0xFF, 0x00]); // full color, no alpha
                }
                mask >>= 1;
            }
        }
        GrayAlphaImage::from_vec(self.bytes_per_line * 8, self.height, buffer).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::chsets::editor::EChar;

    use super::Page;

    #[test]
    fn test_draw_echar_aligned() {
        let chr = EChar {
            width: 8,
            top: 10,
            height: 10,
            buf: &[0xFF; 20],
        };

        let mut page = Page::new(24, 24);
        assert_eq!(&[0u8; 72], &page.buffer[..]);

        // left, top, aligned
        page.draw_echar(0, 0, &chr).unwrap();
        assert_eq!(&[0x00, 0x00, 0], &page.buffer[27..30]);
        assert_eq!(&[0xFF, 0xFF, 0], &page.buffer[30..33]);
        assert_eq!(&[0xFF, 0xFF, 0], &page.buffer[33..36]);
        assert_eq!(&[0xFF, 0xFF, 0], &page.buffer[57..60]);
        assert_eq!(&[0x00, 0x00, 0], &page.buffer[60..63]);
        page.clear();
        assert_eq!(&[0u8; 72], &page.buffer[..]);

        // mid, top, aligned
        page.draw_echar(8, 0, &chr).unwrap();
        assert_eq!(&[0, 0x00, 0x00], &page.buffer[27..30]);
        assert_eq!(&[0, 0xFF, 0xFF], &page.buffer[30..33]);
        assert_eq!(&[0, 0xFF, 0xFF], &page.buffer[33..36]);
        assert_eq!(&[0, 0xFF, 0xFF], &page.buffer[57..60]);
        assert_eq!(&[0, 0x00, 0x00], &page.buffer[60..63]);
        page.clear();
        assert_eq!(&[0u8; 72], &page.buffer[..]);

        // right, top, aligned
        page.draw_echar(16, 0, &chr).unwrap();
        assert_eq!(&[0, 0, 0x00], &page.buffer[27..30]);
        assert_eq!(&[0, 0, 0xFF], &page.buffer[30..33]);
        assert_eq!(&[0, 0, 0xFF], &page.buffer[33..36]);
        assert_eq!(&[0, 0, 0xFF], &page.buffer[57..60]);
        assert_eq!(&[0, 0, 0x00], &page.buffer[60..63]);
        page.clear();
        assert_eq!(&[0u8; 72], &page.buffer[..]);

        // left, bottom, aligned
        page.draw_echar(0, 4, &chr).unwrap();
        assert_eq!(&[0x00, 0x00, 0], &page.buffer[39..42]);
        assert_eq!(&[0xFF, 0xFF, 0], &page.buffer[42..45]);
        assert_eq!(&[0xFF, 0xFF, 0], &page.buffer[63..66]);
        assert_eq!(&[0xFF, 0xFF, 0], &page.buffer[66..69]);
        assert_eq!(&[0xFF, 0xFF, 0], &page.buffer[69..72]);
        page.clear();
        assert_eq!(&[0u8; 72], &page.buffer[..]);

        // mid, bottom, aligned
        page.draw_echar(8, 4, &chr).unwrap();
        assert_eq!(&[0, 0x00, 0x00], &page.buffer[39..42]);
        assert_eq!(&[0, 0xFF, 0xFF], &page.buffer[42..45]);
        assert_eq!(&[0, 0xFF, 0xFF], &page.buffer[63..66]);
        assert_eq!(&[0, 0xFF, 0xFF], &page.buffer[66..69]);
        assert_eq!(&[0, 0xFF, 0xFF], &page.buffer[69..72]);
        page.clear();
        assert_eq!(&[0u8; 72], &page.buffer[..]);

        // right, bottom, aligned
        page.draw_echar(16, 4, &chr).unwrap();
        assert_eq!(&[0, 0, 0x00], &page.buffer[39..42]);
        assert_eq!(&[0, 0, 0xFF], &page.buffer[42..45]);
        assert_eq!(&[0, 0, 0xFF], &page.buffer[63..66]);
        assert_eq!(&[0, 0, 0xFF], &page.buffer[66..69]);
        assert_eq!(&[0, 0, 0xFF], &page.buffer[69..72]);
        page.clear();
        assert_eq!(&[0u8; 72], &page.buffer[..]);
    }

    #[test]
    fn test_draw_echar_unaligned() {
        let chr = EChar {
            width: 12,
            top: 10,
            height: 10,
            buf: &[0xFF; 20],
        };

        let mut page = Page::new(24, 24);
        assert_eq!(&[0u8; 72], &page.buffer[..]);

        // left, top, unaligned
        page.draw_echar(2, 0, &chr).unwrap();
        assert_eq!(&[0x00, 0x00, 0x00], &page.buffer[27..30]);
        assert_eq!(&[0x3F, 0xFF, 0xc0], &page.buffer[30..33]);
        assert_eq!(&[0x3F, 0xFF, 0xc0], &page.buffer[33..36]);
        assert_eq!(&[0x3F, 0xFF, 0xc0], &page.buffer[57..60]);
        assert_eq!(&[0x00, 0x00, 0x00], &page.buffer[60..63]);
        page.clear();
        assert_eq!(&[0u8; 72], &page.buffer[..]);

        // mid, top, unaligned
        page.draw_echar(10, 0, &chr).unwrap();
        assert_eq!(&[0, 0x00, 0x00], &page.buffer[27..30]);
        assert_eq!(&[0, 0x3F, 0xFF], &page.buffer[30..33]);
        assert_eq!(&[0, 0x3F, 0xFF], &page.buffer[33..36]);
        assert_eq!(&[0, 0x3F, 0xFF], &page.buffer[57..60]);
        assert_eq!(&[0, 0x00, 0x00], &page.buffer[60..63]);
        page.clear();
        assert_eq!(&[0u8; 72], &page.buffer[..]);

        // right, top, unaligned
        page.draw_echar(12, 0, &chr).unwrap();
        assert_eq!(&[0, 0x00, 0x00], &page.buffer[27..30]);
        assert_eq!(&[0, 0x0F, 0xFF], &page.buffer[30..33]);
        assert_eq!(&[0, 0x0F, 0xFF], &page.buffer[33..36]);
        assert_eq!(&[0, 0x0F, 0xFF], &page.buffer[57..60]);
        assert_eq!(&[0, 0x00, 0x00], &page.buffer[60..63]);
        page.clear();
        assert_eq!(&[0u8; 72], &page.buffer[..]);

        // left, bottom, unaligned
        page.draw_echar(2, 4, &chr).unwrap();
        assert_eq!(&[0x00, 0x00, 0x00], &page.buffer[39..42]);
        assert_eq!(&[0x3F, 0xFF, 0xc0], &page.buffer[42..45]);
        assert_eq!(&[0x3F, 0xFF, 0xc0], &page.buffer[63..66]);
        assert_eq!(&[0x3F, 0xFF, 0xc0], &page.buffer[66..69]);
        assert_eq!(&[0x3F, 0xFF, 0xc0], &page.buffer[69..72]);
        page.clear();
        assert_eq!(&[0u8; 72], &page.buffer[..]);

        // mid, bottom, unaligned
        page.draw_echar(10, 4, &chr).unwrap();
        assert_eq!(&[0, 0x00, 0x00], &page.buffer[39..42]);
        assert_eq!(&[0, 0x3F, 0xFF], &page.buffer[42..45]);
        assert_eq!(&[0, 0x3F, 0xFF], &page.buffer[63..66]);
        assert_eq!(&[0, 0x3F, 0xFF], &page.buffer[66..69]);
        assert_eq!(&[0, 0x3F, 0xFF], &page.buffer[69..72]);
        page.clear();
        assert_eq!(&[0u8; 72], &page.buffer[..]);

        // right, bottom, unaligned
        page.draw_echar(12, 4, &chr).unwrap();
        assert_eq!(&[0, 0x00, 0x00], &page.buffer[39..42]);
        assert_eq!(&[0, 0x0F, 0xFF], &page.buffer[42..45]);
        assert_eq!(&[0, 0x0F, 0xFF], &page.buffer[63..66]);
        assert_eq!(&[0, 0x0F, 0xFF], &page.buffer[66..69]);
        assert_eq!(&[0, 0x0F, 0xFF], &page.buffer[69..72]);
        page.clear();
        assert_eq!(&[0u8; 72], &page.buffer[..]);
    }
}
