//! # The printer charsets

use super::LoadError;
use crate::util::Buf;
use nom::{
    bytes::complete::{tag, take},
    combinator::verify,
    multi::count,
    number::complete::{be_u32, u8},
    Finish, IResult,
};
use std::{ops::Deref, path::Path};

#[derive(Debug, Copy, Clone)]
/// The supported kinds of printers
pub enum PrinterKind {
    /// A 24-needle printer
    Needle24,
    /// A 9-needle printer
    Needle9,
    /// A laser printer
    Laser30,
}

impl PrinterKind {
    /// Get the extension used for charset files for this printer kind
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Needle24 => "P24",
            Self::Needle9 => "P9",
            Self::Laser30 => "L30",
        }
    }

    /// Get the number of dots for the given amount of horizontal units
    ///
    /// FIXME: this introduces rounding errors
    pub fn scale_x(self, units: u16) -> u32 {
        match self {
            Self::Needle9 => u32::from(units) * 12 / 5,
            Self::Needle24 => u32::from(units) * 4,
            Self::Laser30 => u32::from(units) * 10 / 3,
        }
    }

    /// Get the number of dots for the given amount of vertical units
    ///
    /// FIXME: this introduces rounding errors
    pub fn scale_y(&self, units: u16) -> u32 {
        match self {
            PrinterKind::Needle9 => u32::from(units) * 4,
            PrinterKind::Needle24 => u32::from(units) * 20 / 3,
            PrinterKind::Laser30 => u32::from(units) * 50 / 9,
        }
    }

    /// Get the position of the character baseline from the top of the glyph bounding box
    pub fn baseline(self) -> u8 {
        match self {
            Self::Needle9 => 35,
            Self::Needle24 => 58,
            Self::Laser30 => 48,
        }
    }

    /// Get maximum height of a character
    pub fn line_height(self) -> u32 {
        match self {
            Self::Needle9 => 48,
            Self::Needle24 => 80,
            Self::Laser30 => 68,
        }
    }

    /// Get maximum width of a character
    pub fn max_width(self) -> u8 {
        match self {
            Self::Needle24 => 60,
            Self::Laser30 => 50,
            Self::Needle9 => 40,
        }
    }

    /// The space between the baseline and the bottom of the bounding box (in pixels)
    pub fn descent(self) -> u32 {
        match self {
            Self::Needle9 => 13,
            Self::Needle24 => 22,
            Self::Laser30 => 20,
        }
    }

    /// The default height of the character box (in pixels)
    pub fn ascent(self) -> u32 {
        match self {
            Self::Needle9 => 22,
            Self::Needle24 => 36,
            Self::Laser30 => 30,
        }
    }

    /// Get the resolution of this printer font in dots per inch
    pub fn resolution(&self) -> (u32, u32) {
        match self {
            Self::Needle9 => (240, 216),
            Self::Needle24 => (360, 360),
            Self::Laser30 => (300, 300),
        }
    }
}

#[derive(Debug)]
/// A complete printer charset
pub struct PSet<'a> {
    /// The kind
    pub pk: PrinterKind,
    /// The header
    pub header: Buf<'a>,
    /// The list of characters
    pub chars: Vec<PSetChar<'a>>,
}

#[derive(Debug)]
/// A single printer character
pub struct PSetChar<'a> {
    /// The distance to the top of the line box
    pub top: u8,
    /// The height of the character in pixels
    pub height: u8,
    /// The width of the character in bytes
    pub width: u8,
    /// The pixel data
    pub bitmap: &'a [u8],
}

#[derive(Debug)]
/// A single printer character
pub struct PSetCharBuf {
    /// The distance to the top of the line box
    pub top: u8,
    /// The height of the character in pixels
    pub height: u8,
    /// The width of the character in bytes
    pub width: u8,
    /// The pixel data
    pub bitmap: Vec<u8>,
}

impl PSetCharBuf {
    /// Return a non-owned character
    pub fn as_borrowed(&self) -> PSetChar {
        PSetChar {
            top: self.top,
            height: self.height,
            width: self.width,
            bitmap: &self.bitmap,
        }
    }

    /// Trim the top and bottom
    ///
    /// ```
    /// use signum::chsets::printer::PSetCharBuf;
    ///
    /// let mut p = PSetCharBuf {
    ///     top: 0,
    ///     width: 1,
    ///     height: 8,
    ///     bitmap: vec![
    ///         0b00000000,
    ///         0b00000000,
    ///         0b00000000,
    ///         0b00011000,
    ///         0b00011000,
    ///         0b00000000,
    ///         0b00000000,
    ///         0b00000000,
    ///     ],
    /// };
    /// p.trim_v();
    /// assert_eq!(&p.bitmap, &[
    ///     0b00011000,
    ///     0b00011000,
    /// ]);
    /// assert_eq!(p.top, 3);
    /// ```
    pub fn trim_v(&mut self) {
        let wu = self.width as usize;
        if let Some(len) = self
            .bitmap
            .rchunks(wu)
            .position(|x| x.iter().any(|x| *x > 0))
        {
            let max = self.bitmap.len();
            if len > self.height as usize {
                panic!(
                    "height is {} but len to skip is {}. width={}, max={}",
                    self.height, len, self.width, max
                );
            }
            self.height -= len as u8;
            self.bitmap.drain(max - len * wu..max);
        }
        if let Some(len) = self
            .bitmap
            .chunks(wu)
            .position(|x| x.iter().any(|x| *x > 0))
        {
            self.bitmap.drain(0..len * wu);
            self.top += len as u8;
        }
    }
}

/// A struct to hold information on computed character dimensions
pub struct HBounds {
    /// The number of bits that are zero in every line from the left
    pub max_lead: usize,
    /// The number of bits that are zero in every line from the right
    pub max_tail: usize,
}

const MASK: [u8; 8] = [128, 64, 32, 16, 8, 4, 2, 1];

impl PSetChar<'_> {
    /// Compute the horizontal bounds of the char
    pub fn hbounds(&self) -> HBounds {
        let width = self.width as usize * 8;
        let mut max_lead = width;
        let mut max_tail = width;
        for row in self.bitmap.chunks(self.width as usize) {
            let mut has_bit = false;
            let mut lead: usize = 0;
            let mut tail: usize = 0;
            for byte in row {
                match (has_bit, *byte == 0) {
                    (false, false) => {
                        lead += byte.leading_zeros() as usize;
                        has_bit = true;
                        tail = byte.trailing_zeros() as usize;
                    }
                    (false, true) => {
                        lead += 8;
                    }
                    (true, false) => {
                        tail = byte.trailing_zeros() as usize;
                    }
                    (true, true) => {
                        tail += 8;
                    }
                }
            }
            max_lead = max_lead.min(lead);
            max_tail = max_tail.min(tail);
        }
        HBounds { max_lead, max_tail }
    }

    /// Get whether a specific pixel is set
    ///
    /// ```
    /// use signum::chsets::printer::PSetChar;
    ///
    /// let p = PSetChar {
    ///     width: 1,
    ///     height: 8,
    ///     top: 0,
    ///     bitmap: &[
    ///         0b10000000,
    ///         0b01000000,
    ///         0b00100000,
    ///         0b00010000,
    ///         0b00001000,
    ///         0b00000100,
    ///         0b00000010,
    ///         0b00000001,
    ///     ]
    /// };
    /// for i in 0..8 {
    ///     for j in 0..8 {
    ///         assert_eq!(p.get_ink_at(i, j), i == j);
    ///     }
    /// }
    /// ```
    pub fn get_ink_at(&self, x: usize, y: usize) -> bool {
        let offset = x / 8;
        let wu = self.width as usize;
        if offset >= wu {
            return false;
        }
        let tu = self.top as usize;
        if y < tu {
            return false;
        }
        let hu = self.height as usize;
        let yoff = y - tu;
        if yoff >= hu {
            return false;
        }
        let byte = self.bitmap[yoff * wu + offset];
        let mask = MASK[x % 8];
        (mask & byte) > 0
    }

    /// Get the number of pixels in a 3x3 grid around the specified coordiante
    pub fn kernel3x3(&self, x: usize, y: usize) -> u8 {
        let mut count = 0;
        if y > 0 {
            if x > 0 && self.get_ink_at(x - 1, y - 1) {
                count += 1;
            }
            if self.get_ink_at(x, y - 1) {
                count += 1;
            }
            if self.get_ink_at(x + 1, y - 1) {
                count += 1;
            }
        }
        if x > 0 {
            if self.get_ink_at(x - 1, y) {
                count += 1;
            }
            if self.get_ink_at(x - 1, y + 1) {
                count += 1;
            }
        }
        if self.get_ink_at(x, y) {
            count += 1;
        }
        if self.get_ink_at(x + 1, y) {
            count += 1;
        }
        if self.get_ink_at(x, y + 1) {
            count += 1;
        }
        if self.get_ink_at(x + 1, y + 1) {
            count += 1;
        }
        count
    }

    /// Make a fake-bold variant of the glyph
    ///
    /// ```
    /// use signum::chsets::printer::PSetChar;
    ///
    /// let b0 = &[
    ///     0b00000000,
    ///     0b00000000,
    ///     0b00000000,
    ///     0b00011000,
    ///     0b00011000,
    ///     0b00000000,
    ///     0b00000000,
    ///     0b00000000,
    /// ];
    /// let b1 = &[
    ///     0b00000000,
    ///     0b00000000,
    ///     0b00111100,
    ///     0b00111100,
    ///     0b00111100,
    ///     0b00111100,
    ///     0b00000000,
    ///     0b00000000,
    /// ];
    ///
    /// let p = PSetChar {
    ///     width: 1,
    ///     height: 8,
    ///     top: 0,
    ///     bitmap: b0,
    /// };
    /// let p2 = p.fakebold(1, 8);
    /// assert_eq!(&p2.bitmap, b1);
    /// ```
    pub fn fakebold(&self, width: u8, height: u8) -> PSetCharBuf {
        let wu = width as usize;
        let hu = height as usize;
        let mut bitmap = Vec::with_capacity(wu * hu);

        for y in 0..hu {
            for x in 0..wu {
                let mut byte = 0;
                for (off, m) in IntoIterator::into_iter(MASK).enumerate() {
                    let count = self.kernel3x3(x * 8 + off, y);
                    if count > 0 {
                        byte |= m;
                    }
                }
                bitmap.push(byte);
            }
        }

        let mut out = PSetCharBuf {
            top: 0,
            height,
            width,
            bitmap,
        };
        out.trim_v();
        out
    }
}

/// An owned printer character set
pub struct OwnedPSet {
    inner: PSet<'static>,
    #[allow(unused)]
    buffer: Vec<u8>,
}

impl Deref for OwnedPSet {
    type Target = PSet<'static>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl OwnedPSet {
    /// Load a character set
    pub fn load(path: &Path, kind: PrinterKind) -> Result<Self, LoadError> {
        let buffer = std::fs::read(path)?;
        // SAFETY: this is safe, because `buffer` is plain data and
        // drop order between `inner` and `buffer` really doesn't matter.
        let input: &'static [u8] = unsafe { std::mem::transmute(&buffer[..]) };
        let (_, inner) = match kind {
            PrinterKind::Needle24 => parse_ps24(input),
            PrinterKind::Laser30 => parse_ls30(input),
            PrinterKind::Needle9 => parse_ps09(input),
        }
        .finish()
        .map_err(|e| LoadError::Parse(format!("{:?}", e)))?;
        Ok(Self { inner, buffer })
    }
}

/// Parse a single P24 character
pub fn parse_char(input: &[u8]) -> IResult<&[u8], PSetChar> {
    let (input, top) = u8(input)?;
    let (input, height) = u8(input)?;
    let (input, width) = u8(input)?;
    // FIXME: Are there any valid files where this is non-zero?
    let (input, _d) = verify(u8, |x| *x == 0)(input)?;

    let len = (width as usize) * (height as usize);
    let (input, bitmap) = take(len)(input)?;
    let input = if len % 2 == 1 { &input[1..] } else { input };

    Ok((
        input,
        PSetChar {
            top,
            height,
            width,
            bitmap,
        },
    ))
}

/// Parse a a font file
///
/// This method only checks the `0001` part of the magic bytes
pub fn parse_font(input: &[u8], pk: PrinterKind) -> IResult<&[u8], PSet> {
    let (input, _) = tag(b"0001")(input)?;
    let (input, _) = verify(be_u32, |x| *x == 128)(input)?;

    let (input, header) = take(128usize)(input)?;
    let (input, _len) = be_u32(input)?;

    let (input, _offset_buf) = take(127usize * 4)(input)?;
    let (input, chars) = count(parse_char, 128usize)(input)?;

    Ok((
        input,
        PSet {
            pk,
            header: Buf(header),
            chars,
        },
    ))
}

/// Parse a P24 file
pub fn parse_ps24(input: &[u8]) -> IResult<&[u8], PSet> {
    let (input, _) = tag(b"ps24")(input)?;
    parse_font(input, PrinterKind::Needle24)
}

/// Parse a P09 file
pub fn parse_ps09(input: &[u8]) -> IResult<&[u8], PSet> {
    let (input, _) = tag(b"ps09")(input)?;
    parse_font(input, PrinterKind::Needle9)
}

/// Parse a L30 file
pub fn parse_ls30(input: &[u8]) -> IResult<&[u8], PSet> {
    let (input, _) = tag(b"ls30")(input)?;
    parse_font(input, PrinterKind::Laser30)
}
