use super::LoadError;
use crate::util::Buf;
use nom::{
    bytes::complete::{tag, take},
    combinator::verify,
    multi::count,
    number::complete::{be_u32, be_u8},
    IResult,
};
use std::{ops::Deref, path::Path};

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
            }, // TODO
        }
        .unwrap();
        Ok(Self { inner, buffer })
    }
}

pub fn parse_ps24_char(input: &[u8]) -> IResult<&[u8], PSetChar> {
    let (input, top) = be_u8(input)?;
    let (input, height) = be_u8(input)?;
    let (input, width) = be_u8(input)?;
    let (input, _d) = be_u8(input)?;
    assert_eq!(_d, 0);
    let count = {
        let len = (width as usize) * (height as usize);
        if len % 2 == 0 {
            len
        } else {
            len + 1
        }
    };
    let (input, bitmap) = take(count)(input)?;

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
