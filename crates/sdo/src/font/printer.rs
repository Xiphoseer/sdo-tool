use super::LoadError;
use crate::util::Buf;
use nom::{
    bytes::complete::{tag, take},
    combinator::verify,
    multi::count,
    number::complete::{be_u32, u8},
    IResult,
};
use std::{ops::Deref, path::Path, str::FromStr};
use thiserror::*;

#[derive(Debug, Copy, Clone)]
pub enum PrintDriver {
    Editor,
    Printer9,
    Printer24,
    Laser30,
}

impl PrintDriver {
    pub fn scale_y(&self, units: u16) -> u32 {
        match self {
            Self::Editor => u32::from(units) * 2,
            Self::Printer9 => u32::from(units) * 4,
            Self::Printer24 => u32::from(units) * 20 / 3,
            Self::Laser30 => u32::from(units) * 50 / 9,
        }
    }

    pub fn scale_x(&self, units: u16) -> u32 {
        match self {
            Self::Editor => u32::from(units),
            Self::Printer9 => u32::from(units) * 12 / 5,
            Self::Printer24 => u32::from(units) * 4,
            Self::Laser30 => u32::from(units) * 10 / 3,
        }
    }

    pub fn resolution(&self) -> (isize, isize) {
        match self {
            Self::Editor => (104, 90),
            Self::Printer9 => (216, 216),
            Self::Printer24 => (360, 360),
            Self::Laser30 => (300, 300),
        }
    }
}

#[derive(Debug, Error)]
#[error("Unknown print driver!")]
pub struct UnknownPrintDriver {}

impl FromStr for PrintDriver {
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "E24" => Ok(Self::Editor),
            "P24" => Ok(Self::Printer24),
            "L30" => Ok(Self::Laser30),
            _ => Err(UnknownPrintDriver {}),
        }
    }

    type Err = UnknownPrintDriver;
}

#[derive(Debug, Copy, Clone)]
pub enum FontKind {
    Needle24,
    Needle9,
    Laser30,
}

impl FontKind {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Needle24 => "P24",
            Self::Needle9 => "P9",
            Self::Laser30 => "L30",
        }
    }
}

#[derive(Debug)]
pub struct PSet<'a> {
    pub header: Buf<'a>,
    pub chars: Vec<PSetChar<'a>>,
}

#[derive(Debug)]
pub struct PSetChar<'a> {
    pub top: u8,
    pub height: u8,
    pub width: u8,
    pub bitmap: &'a [u8],
}

pub struct HBounds {
    pub max_lead: usize,
    pub max_tail: usize,
}

impl PSetChar<'_> {
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

pub struct OwnedPSet {
    inner: PSet<'static>,
    buffer: Vec<u8>,
}

impl Deref for OwnedPSet {
    type Target = PSet<'static>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl OwnedPSet {
    pub fn load(path: &Path, kind: FontKind) -> Result<Self, LoadError> {
        let buffer = std::fs::read(path)?;
        // SAFETY: this is safe, because `buffer` is plain data and
        // drop order between `inner` and `buffer` really doesn't matter.
        let input: &'static [u8] = unsafe { std::mem::transmute(&buffer[..]) };
        let (_, inner) = match kind {
            FontKind::Needle24 => parse_ps24(input),
            FontKind::Laser30 => parse_ls30(input),
            FontKind::Needle9 => {
                return Err(LoadError::Unimplemented);
            } // TODO
        }
        .unwrap();
        Ok(Self { inner, buffer })
    }
}

pub fn parse_ps24_char(input: &[u8]) -> IResult<&[u8], PSetChar> {
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

pub fn parse_font(input: &[u8]) -> IResult<&[u8], PSet> {
    let (input, _) = tag(b"0001")(input)?;
    let (input, _) = verify(be_u32, |x| *x == 128)(input)?;

    let (input, header) = take(128usize)(input)?;
    let (input, _len) = be_u32(input)?;

    let (input, _offset_buf) = take(127usize * 4)(input)?;
    let (input, chars) = count(parse_ps24_char, 128usize)(input)?;

    Ok((
        input,
        PSet {
            header: Buf(header),
            chars,
        },
    ))
}

pub fn parse_ps24(input: &[u8]) -> IResult<&[u8], PSet> {
    let (input, _) = tag(b"ps24")(input)?;
    parse_font(input)
}

pub fn parse_ls30(input: &[u8]) -> IResult<&[u8], PSet> {
    let (input, _) = tag(b"ls30")(input)?;
    parse_font(input)
}
