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

    /// Get the scale factor
    pub fn scale(&self) -> f32 {
        match self {
            Self::Needle9 => todo!(),
            Self::Needle24 => 0.2,
            Self::Laser30 => 0.24,
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
    pub fn baseline(self) -> i16 {
        match self {
            Self::Needle9 => 36,
            Self::Needle24 => 58,
            Self::Laser30 => 48,
        }
    }
}

#[derive(Debug)]
/// A complete printer charset
pub struct PSet<'a> {
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

/// A struct to hold information on computed character dimensions
pub struct HBounds {
    /// The number of bits that are zero in every line from the left
    pub max_lead: usize,
    /// The number of bits that are zero in every line from the right
    pub max_tail: usize,
}

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
    let (input, _d) = u8(input)?;
    assert_eq!(_d, 0);
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
pub fn parse_font(input: &[u8]) -> IResult<&[u8], PSet> {
    let (input, _) = tag(b"0001")(input)?;
    let (input, _) = verify(be_u32, |x| *x == 128)(input)?;

    let (input, header) = take(128usize)(input)?;
    let (input, _len) = be_u32(input)?;

    let (input, _offset_buf) = take(127usize * 4)(input)?;
    let (input, chars) = count(parse_char, 128usize)(input)?;

    Ok((
        input,
        PSet {
            header: Buf(header),
            chars,
        },
    ))
}

/// Parse a P24 file
pub fn parse_ps24(input: &[u8]) -> IResult<&[u8], PSet> {
    let (input, _) = tag(b"ps24")(input)?;
    parse_font(input)
}

/// Parse a P09 file
pub fn parse_ps09(input: &[u8]) -> IResult<&[u8], PSet> {
    let (input, _) = tag(b"ps09")(input)?;
    parse_font(input)
}

/// Parse a L30 file
pub fn parse_ls30(input: &[u8]) -> IResult<&[u8], PSet> {
    let (input, _) = tag(b"ls30")(input)?;
    parse_font(input)
}
