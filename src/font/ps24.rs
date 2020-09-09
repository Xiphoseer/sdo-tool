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
    pub fn load(path: &Path) -> Result<Self, LoadError> {
        let buffer = std::fs::read(path)?;
        // SAFETY: this is safe, because `buffer` is plain data and
        // drop order between `inner` and `buffer` really doesn't matter.
        let input: &'static [u8] = unsafe { std::mem::transmute(&buffer[..]) };
        let (_, inner) = parse_ps24(input).unwrap();
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

pub fn parse_ps24(input: &[u8]) -> IResult<&[u8], PSet> {
    let (input, _) = tag(b"ps24")(input)?;
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
