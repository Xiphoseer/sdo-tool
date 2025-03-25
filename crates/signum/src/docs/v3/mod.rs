//! # Signum! 3/4 Documents
//!
//! Signum! 3/4 uses a completely different format from 1/2.

use core::fmt;
use std::{borrow::Cow, fmt::Write, num::NonZeroU32};

use bstr::BStr;
use nom::{
    bytes::complete::{tag, take, take_while},
    combinator::{cond, map, map_parser, rest, verify},
    error::{context, ContextError, ParseError},
    multi::{length_value, many0, many1, many_m_n},
    number::complete::{be_i16, be_u16, be_u32, u8},
    sequence::{pair, preceded, terminated, tuple},
    IResult, Parser,
};

use crate::{
    chsets::encoding::decode_atari_str,
    util::{Buf, Bytes32},
};

use super::bytes32;

/// Tag for a v3 document
pub const TAG_SDOC3: &[u8; 12] = b"\0\0sdoc  03\0\0";

/// Tag for *file pointers*
pub const CHUNK_FLPTRS01: &[u8; 12] = b"\0\0flptrs01\0\0";
/// Tag for *fonts used*
pub const CHUNK_FOUSED01: &[u8; 12] = b"\0\0foused01\0\0";
/// Tag for *chapter*
pub const CHUNK_KAPIT01: &[u8; 12] = b"\0\0kapit 01\0\0";
/// Tag for *stream*
pub const CHUNK_STREAM01: &[u8; 12] = b"\0\0stream01\0\0";

/// Header of a document
#[derive(Debug)]
#[allow(dead_code)]
pub struct Header<'a> {
    lead: Buf<'a>,
    /// Create time
    pub ctime: DateTime,
    /// Modified time
    pub mtime: DateTime,
    tail: Buf<'a>,
}

/// Document root
#[allow(dead_code)]
#[derive(Debug)]
pub struct SDocV3<'a> {
    /// Header buffer
    header: Header<'a>,
    /// The file pointers
    file_pointers: FilePointers,
    /// The fonts in the file
    fonts: FontsUsed<'a>,
    /// The chapters in the documents
    chapters: Vec<Chapter<'a>>,
}

impl<'a> SDocV3<'a> {
    /// Get the *file pointers* `flptrs01` chunk
    pub fn sdoc03(&self) -> &Header {
        &self.header
    }

    /// Get the *file pointers* `flptrs01` chunk
    pub fn flptrs01(&self) -> &FilePointers {
        &self.file_pointers
    }

    /// Get the *fonts used* `foused01` chunk
    pub fn foused01(&self) -> &FontsUsed<'a> {
        &self.fonts
    }

    /// Get the *chapters* `kapit 01` + `stream01` chunks
    pub fn chapters(&self) -> &[Chapter<'a>] {
        &self.chapters
    }
}

/// `flptrs01` chunk
#[allow(dead_code)]
#[derive(Debug)]
pub struct FilePointers {
    /// Offset of `foused01`
    pub ofs_foused01: u32,
    /// Offset of `params01`
    pub ofs_params01: u32,
    /// Offset of `cdilist0`
    pub ofs_cdilist0: Option<NonZeroU32>,
    /// Offset of `foxlist `
    pub ofs_foxlist: Option<NonZeroU32>,
    u5: Option<NonZeroU32>,
    u6: Option<NonZeroU32>,
    u7: Option<NonZeroU32>,
    u8: Option<NonZeroU32>,
    /// Offsets of `kapit 01`
    pub ofs_chapters: Vec<u32>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Full date
pub struct Date {
    year: u16,
    month: u16,
    day: u16,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Time of day
pub struct Time {
    hour: u16,
    minute: u16,
    second: u16,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// [Date] and [Time]
pub struct DateTime {
    date: Date,
    time: Time,
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.date.fmt(f)?;
        f.write_str("T")?;
        self.time.fmt(f)?;
        Ok(())
    }
}

fn parse_date<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Date, E> {
    let (input, (year, month, day)) = tuple((be_u16, be_u16, be_u16))(input)?;
    Ok((input, Date { year, month, day }))
}

fn parse_time<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Time, E> {
    let (input, (hour, minute, second)) = tuple((be_u16, be_u16, be_u16))(input)?;
    Ok((
        input,
        Time {
            hour,
            minute,
            second,
        },
    ))
}

fn parse_datetime<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], DateTime, E> {
    let (input, (date, time)) = pair(parse_date, parse_time)(input)?;
    Ok((input, DateTime { date, time }))
}

fn parse_header<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Header<'a>, E> {
    let (input, lead) = map(take(40usize), Buf)(input)?;
    let (input, ctime) = parse_datetime(input)?;
    let (input, mtime) = parse_datetime(input)?;
    let (input, tail) = map(rest, Buf)(input)?;
    Ok((
        input,
        Header {
            lead,
            ctime,
            mtime,
            tail,
        },
    ))
}

fn opt_be_nonzero_u32<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Option<NonZeroU32>, E> {
    let (input, val) = be_u32(input)?;
    Ok((input, NonZeroU32::new(val)))
}

/// A single used font
pub type FontUsed<'a> = (u8, Cow<'a, str>);

/// `foused01` chunk
#[derive(Debug)]
pub struct FontsUsed<'a> {
    fonts: Vec<FontUsed<'a>>,
}

impl<'a> FontsUsed<'a> {
    /// Get all fonts
    pub fn fonts(&self) -> &[FontUsed<'a>] {
        &self.fonts
    }
}

fn parse_font_used<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], FontUsed<'a>, E> {
    let (input, index) = nom::number::complete::u8(input)?;
    let (input, name) = map_parser(take(9usize), take_while(|c| c != 0u8))(input)?;
    Ok((input, (index, decode_atari_str(name))))
}

/// Parse a `foused01` chunk
pub fn parse_foused01<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], FontsUsed<'a>, E> {
    map(many0(parse_font_used), |fonts| FontsUsed { fonts })(input)
}

fn parse_flptrs01<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], FilePointers, E> {
    let (input, ofs_foused01) = verify(be_u32, |v| *v > 0)(input)?;
    let (input, ofs_params01) = verify(be_u32, |v| *v > 0)(input)?;
    let (input, ofs_cdilist0) = opt_be_nonzero_u32(input)?;
    let (input, ofs_foxlist) = opt_be_nonzero_u32(input)?;
    let (input, u5) = opt_be_nonzero_u32(input)?;
    let (input, u6) = opt_be_nonzero_u32(input)?;
    let (input, u7) = opt_be_nonzero_u32(input)?;
    let (input, u8) = opt_be_nonzero_u32(input)?;
    let (input, ofs_chapters) = many1(be_u32)(input)?;
    Ok((
        input,
        FilePointers {
            ofs_foused01,
            ofs_params01,
            ofs_cdilist0,
            ofs_foxlist,
            u5,
            u6,
            u7,
            u8,
            ofs_chapters,
        },
    ))
}

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

fn parse_stream01<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Stream<'a>, E> {
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

fn parse_stream_chunk<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Stream<'a>, E>
where
    E: ContextError<&'a [u8]>,
    E: ParseError<&'a [u8]>,
{
    context("stream01", parse_chunk_head(CHUNK_STREAM01, parse_stream01))(input)
}

/// A *chapter* (`kapit 01` and `stream01` chunks)
#[derive(Debug)]
#[allow(dead_code)]
pub struct Chapter<'a> {
    header: ChapterHeader<'a>,
    main: Stream<'a>,
    head_foot: Stream<'a>,
    s3: Option<Stream<'a>>,
    s4: Option<Stream<'a>>,
}

impl<'a> Chapter<'a> {
    /// Get the header of a chapter
    pub fn header(&self) -> &ChapterHeader<'a> {
        &self.header
    }

    /// Return the main `stream01`
    pub fn main(&self) -> &Stream<'a> {
        &self.main
    }

    /// Return the main `stream01`
    pub fn header_footer(&self) -> &Stream<'a> {
        &self.head_foot
    }

    /// Return the 3rd `stream01`
    pub fn stream3(&self) -> Option<&Stream<'a>> {
        self.s3.as_ref()
    }

    /// Return the 4th `stream01`, if present
    pub fn stream4(&self) -> Option<&Stream<'a>> {
        self.s4.as_ref()
    }
}

/// Chunk *chapter* header `kapit 01`
#[allow(dead_code)]
#[derive(Debug)]
pub struct ChapterHeader<'a> {
    v1: u16,
    len1: u32,
    v3: u16,
    v4: [i16; 4],
    v8: i16,
    v9: i16,
    v10: i16,
    buf1: Buf<'a>, // len 66
    buf: Buf<'a>,  // size: len1
}

fn parse_kapit01<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], ChapterHeader<'a>, E> {
    let (input, v1) = be_u16(input)?;
    let (input, len1) = be_u32(input)?;
    let (input, v3) = be_u16(input)?;
    let (input, v4) = be_i16(input)?;
    let (input, v5) = be_i16(input)?;
    let (input, v6) = be_i16(input)?;
    let (input, v7) = be_i16(input)?;
    let (input, v8) = be_i16(input)?;
    let (input, v9) = be_i16(input)?;
    let (input, v10) = be_i16(input)?;
    let v4 = [v4, v5, v6, v7];
    let (input, buf1) = map_buf(take(66usize))(input)?;
    let (input, buf) = map_buf(rest)(input)?;

    Ok((
        input,
        ChapterHeader {
            v1,
            len1,
            v3,
            v4,
            v8,
            v9,
            v10,
            buf1,
            buf,
        },
    ))
}

/// Parse a chunk
fn map_buf<'a, F, E: ParseError<&'a [u8]>>(
    parser: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Buf<'a>, E>
where
    F: Parser<&'a [u8], &'a [u8], E>,
{
    map(parser, Buf)
}

/// Parse a chunk
#[allow(dead_code)]
fn map_bstr<'a, F, E: ParseError<&'a [u8]>>(
    parser: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], &'a BStr, E>
where
    F: Parser<&'a [u8], &'a [u8], E>,
{
    map(parser, BStr::new)
}

/// Parse a chunk
fn parse_chunk_head<'a, T, F, E: ParseError<&'a [u8]>>(
    chunk_tag: &'static [u8; 12],
    parser: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], T, E>
where
    F: Parser<&'a [u8], T, E>,
{
    let p = length_value(terminated(be_u16, be_u16), parser);
    preceded(tag(chunk_tag), p)
}

/// Parse a Signum! document
pub fn parse_sdoc_v3<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], SDocV3<'a>, E>
where
    E: ParseError<&'a [u8]>,
    E: ContextError<&'a [u8]>,
{
    let data = input;
    let (input, header) = parse_chunk_head(TAG_SDOC3, parse_header)(input)?;
    let (input, file_pointers) = parse_chunk_head(CHUNK_FLPTRS01, parse_flptrs01)(input)?;

    let (_, fonts) = {
        let (i_foused01, _) = take(file_pointers.ofs_foused01)(data)?;
        context("foused01", parse_chunk_head(CHUNK_FOUSED01, parse_foused01))(i_foused01)
    }?;

    let mut chapters = Vec::new();
    for &ofs_kapit in &file_pointers.ofs_chapters {
        let (input, _) = take(ofs_kapit)(data)?;
        let (input, kapit) =
            context("kapit01", parse_chunk_head(CHUNK_KAPIT01, parse_kapit01))(input)?;
        let (input, main) = parse_stream_chunk(input)?;
        let (input, head_foot) = parse_stream_chunk(input)?;
        let (input, s3) = cond(kapit.v9 >= 0, parse_stream_chunk)(input)?;
        let (input, s4) = cond(kapit.v10 >= 0, parse_stream_chunk)(input)?;
        let _ = input;
        chapters.push(Chapter {
            header: kapit,
            main,
            head_foot,
            s3,
            s4,
        })
    }

    Ok((
        input,
        SDocV3 {
            header,
            file_pointers,
            fonts,
            chapters,
        },
    ))
}
