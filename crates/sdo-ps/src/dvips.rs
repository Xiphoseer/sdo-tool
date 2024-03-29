//! This module reads a Bitmap font embedded in a PostScript file as generated by
//! `dvips -V`, which appears in the source with a comment of `DVIPSBitmapFont`
use std::str::from_utf8_unchecked;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::multispace0,
    character::complete::multispace1,
    character::complete::{char, digit1, hex_digit1},
    combinator::map,
    combinator::map_res,
    combinator::value,
    multi::count,
    multi::separated_list1,
    number::complete::u8,
    sequence::delimited,
    sequence::preceded,
    sequence::terminated,
    IResult,
};

use signum::chsets::printer::PSetChar;

#[derive(Debug)]
pub struct Font {
    pub len: usize,
    pub max: usize,
    pub chars: Vec<Char>,
}

fn from_dec(src: &str) -> Result<usize, std::num::ParseIntError> {
    src.parse::<usize>()
}

fn parse_usize(input: &[u8]) -> IResult<&[u8], usize> {
    map_res(digit1, |b| {
        let src = unsafe { from_utf8_unchecked(b) };
        from_dec(src)
    })(input)
}

#[derive(Debug)]
pub struct Stream {
    pub inner: Vec<u8>,
}

pub fn parse_stream(input: &[u8]) -> IResult<&[u8], Stream> {
    let (input, parts): (&[u8], Vec<&[u8]>) = delimited(
        char('<'),
        separated_list1(multispace1, hex_digit1),
        char('>'),
    )(input)?;

    let mut iter = parts.into_iter().flat_map(|b| b.iter()).cloned();
    let mut inner = vec![];
    while let Some(first) = iter.next() {
        let second = iter.next().unwrap();
        let local = [first, second];
        let src = unsafe { std::str::from_utf8_unchecked(&local) };
        let byte = u8::from_str_radix(src, 16).unwrap();
        inner.push(byte);
    }

    Ok((input, Stream { inner }))
}

#[derive(Copy, Clone, Debug)]
pub enum Cmd {
    Inc,
    Digits(usize),
}

pub fn parse_cmd(input: &[u8]) -> IResult<&[u8], Cmd> {
    alt((
        value(Cmd::Inc, char('I')),
        map(
            terminated(parse_usize, preceded(multispace1, char('D'))),
            Cmd::Digits,
        ),
    ))(input)
}

#[derive(Debug)]
pub struct Char {
    pub stream: Stream,
    pub cmd: Cmd,
}

pub fn parse_char(input: &[u8]) -> IResult<&[u8], Char> {
    let (input, _) = multispace0(input)?;
    let (input, stream) = parse_stream(input)?;
    let (input, _) = multispace0(input)?;
    let (input, cmd) = parse_cmd(input)?;
    Ok((input, Char { stream, cmd }))
}

#[derive(Debug, Copy, Clone)]
pub struct CharHeader {
    pub width: u8,
    pub height: u8,
    pub x_offset: u8,
    pub y_offset: u8,
    pub delta_x: u8,
}

pub struct CharHeaderIter<'a> {
    ch: &'a CharHeader,
    st: usize,
}

impl Iterator for CharHeaderIter<'_> {
    type Item = u8;

    #[rustfmt::skip]
    fn next(&mut self) -> Option<u8> {
        match self.st {
            0 => { self.st = 1; Some(self.ch.width) },
            1 => { self.st = 2; Some(self.ch.height) },
            2 => { self.st = 3; Some(self.ch.x_offset) },
            3 => { self.st = 4; Some(self.ch.y_offset) },
            4 => { self.st = 5; Some(self.ch.delta_x) },
            _ => None,
        }
    }
}

impl CharHeader {
    pub fn iter(&self) -> CharHeaderIter {
        CharHeaderIter { ch: self, st: 0 }
    }

    pub fn from_signum(p: &PSetChar) -> CharHeader {
        let baseline = 32;
        let left = 0;
        CharHeader {
            width: p.width << 3,
            height: p.height,
            y_offset: 127 + baseline - p.top,
            x_offset: 128 - left,
            delta_x: p.width << 3,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CacheDevice {
    pub w_x: i16,
    pub w_y: i16,
    pub ll_x: i32,
    pub ll_y: i32,
    pub ur_x: i32,
    pub ur_y: i32,
}

impl From<CharHeader> for CacheDevice {
    fn from(c: CharHeader) -> CacheDevice {
        let ll_x = 128i32 - (c.x_offset as i32);
        let ur_y = (c.y_offset as i32) - 127;
        CacheDevice {
            w_x: c.delta_x as i16,
            w_y: 0,
            ll_x,
            ll_y: ur_y - (c.height as i32),
            ur_x: ll_x + (c.width as i32),
            ur_y,
        }
    }
}

impl From<CacheDevice> for CharHeader {
    fn from(c: CacheDevice) -> CharHeader {
        CharHeader {
            width: (c.ur_x - c.ll_x) as u8,
            height: (c.ur_y - c.ll_y) as u8,
            y_offset: (c.ur_y + 127) as u8,
            x_offset: (128 - c.ll_x) as u8,
            delta_x: c.w_x as u8,
        }
    }
}

pub fn parse_char_header(input: &[u8]) -> IResult<&[u8], CharHeader> {
    let (input, width) = u8(input)?;
    let (input, height) = u8(input)?;
    let (input, x_offset) = u8(input)?;
    let (input, y_offset) = u8(input)?;
    let (input, delta_x) = u8(input)?;
    Ok((
        input,
        CharHeader {
            width,
            height,
            x_offset,
            y_offset,
            delta_x,
        },
    ))
}

pub fn parse_dvips_bitmap_font(input: &[u8]) -> IResult<&[u8], Font> {
    let (input, _) = tag(b"/Fa")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, len) = parse_usize(input)?;
    let (input, _) = multispace1(input)?;
    let (input, max) = parse_usize(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = tag(b"df")(input)?;
    let (input, chars) = count(parse_char, len)(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = char('E')(input)?;

    Ok((input, Font { len, max, chars }))
}
