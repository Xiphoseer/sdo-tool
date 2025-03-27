use std::fmt::{self, Write};

use nom::{
    bytes::complete::take,
    combinator::{map, map_parser},
    error::ParseError,
    multi::{many0, many_m_n},
    number::complete::{be_u16, be_u32, u8},
    IResult,
};

use crate::{
    docs::bytes32,
    util::{map_buf, Buf, Bytes32, V3Chunk},
};

/// A *data stream* `stream01`
#[allow(dead_code)]
#[derive(Debug)]
pub struct Stream<'a> {
    index: u16,
    uv1: u16,
    count: u16,
    u2: u16,
    v1: u16,
    v2: u16,
    len1: u16,
    v4: u16,
    len3: u32,
    ofs2: u32,
    len2: u32,
    iter: Vec<Buf<'a>>, // repeat: count, size: 8
    buf0: Buf<'a>,      // size: len1
    buf1: Buf<'a>,      // size: len2
    text: Vec<Line>,    // size: len3
    after: Bytes32,
}

impl Stream<'_> {
    /// Get the text data
    pub fn text(&self) -> &[Line] {
        &self.text
    }
}

/// A single character
#[derive(Debug)]
#[allow(dead_code)]
pub struct TChar(u8, u16, u8);

#[derive(Default)]
struct Chars(Vec<TChar>);

impl fmt::Debug for Chars {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('"')?;
        for char in &self.0 {
            let c = char::from(char.2);
            if char.0 == 0x90 || char.0 == 0x98 {
                // pair kerning, don't output
            } else if char.0 == 0x80 {
                // some setting
                write!(f, "\\w{:04X}", char.1)?;
                if char.1 == 0 {
                    continue;
                }
            } else if char.0 & 0x80 > 0 {
                write!(f, "\\e{:02x},{:04x}", char.0, char.1)?;
            }
            if char.2 == 0 {
                f.write_char(' ')?;
            } else if c.is_ascii_graphic() {
                f.write_char(c)?;
            } else {
                write!(f, "\\x{:02x}", char.2)?;
            }
        }
        f.write_char('"')?;
        Ok(())
    }
}

/// A single line in the text
#[derive(Debug)]
#[allow(dead_code)]
pub struct Line {
    vskip: u16,
    chars: Chars,
}

fn parse_tchar<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], TChar, E>
where
    E: ParseError<&'a [u8]>,
{
    let (input, byte) = u8(input)?;
    let (input, extra) = if byte & 0x80 > 0 {
        be_u16(input)?
    } else {
        (input, 0)
    };
    let (input, byte2) = u8(input)?;
    Ok((input, TChar(byte, extra, byte2)))
}

impl fmt::Display for TChar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <char as fmt::Display>::fmt(&char::from(self.2), f)
    }
}

fn parse_line<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Line, E>
where
    E: ParseError<&'a [u8]>,
{
    let (input, len) = be_u16(input)?;
    if len >= 2 {
        let (input, vskip) = be_u16(input)?;
        let (input, data) = if len >= 4 {
            let (input, data) = take(len - 4)(input)?;
            let (_, chars) = map(many0(parse_tchar), Chars)(data)?;
            (input, chars)
        } else {
            (input, Chars::default())
        };
        Ok((input, Line { vskip, chars: data }))
    } else {
        Ok((
            input,
            Line {
                vskip: 0,
                chars: Chars(Vec::new()),
            },
        ))
    }
}

/// Tag for *stream*
pub const CHUNK_STREAM01: &[u8; 12] = b"\0\0stream01\0\0";

impl<'a> V3Chunk<'a> for Stream<'a> {
    const CONTEXT: &'static str = "stream01";

    const TAG: &'static [u8; 12] = CHUNK_STREAM01;

    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>,
    {
        let (input, index) = be_u16(input)?;
        let (input, uv1) = be_u16(input)?;
        let (input, count) = be_u16(input)?;
        let (input, u2) = be_u16(input)?;
        let (input, v1) = be_u16(input)?;
        let (input, v2) = be_u16(input)?;
        let (input, len1) = be_u16(input)?;
        let (input, v4) = be_u16(input)?;
        let (input, len3) = be_u32(input)?;
        let (input, ofs2) = be_u32(input)?;
        let (input, len2) = be_u32(input)?;
        let cu = count as usize;
        let (input, iter) = many_m_n(cu, cu, map_buf(take(8usize)))(input)?;
        let (input, buf0) = map_buf(take(len1))(input)?;
        let (input, buf1) = map_buf(take(len2))(input)?;
        let (input, after) = bytes32(input)?;
        let (input, text) = map_parser(take(len3), many0(parse_line))(input)?;
        Ok((
            input,
            Stream {
                index,
                uv1,
                count,
                u2,
                v1,
                v2,
                len1,
                v4,
                len3,
                ofs2,
                len2,
                iter,
                buf0,
                buf1,
                text,
                after,
            },
        ))
    }
}
