//! # Signum! 3/4 Documents
//!
//! Signum! 3/4 uses a completely different format from 1/2.

use std::{borrow::Cow, num::NonZeroU32};

use nom::{
    bytes::complete::{tag, take, take_while},
    combinator::{map, map_parser, rest, verify},
    error::{context, ContextError, ParseError},
    multi::{length_value, many0, many1},
    number::complete::{be_u16, be_u32},
    sequence::{preceded, terminated},
    IResult, Parser,
};

use crate::{chsets::encoding::decode_atari_str, util::Buf};

/// Tag for a v3 document
pub const TAG_SDOC3: &[u8; 12] = b"\0\0sdoc  03\0\0";

/// Tag for *file pointers*
pub const CHUNK_FLPTRS01: &[u8; 12] = b"\0\0flptrs01\0\0";
/// Tag for *fonts used*
pub const CHUNK_FOUSED01: &[u8; 12] = b"\0\0foused01\0\0";

/// Header of a document
#[derive(Debug)]
#[allow(dead_code)]
pub struct Header<'a> {
    buf: Buf<'a>,
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
}

impl<'a> SDocV3<'a> {
    /// Get the *file pointers* `flptrs01` chunk
    pub fn flptrs01(&self) -> &FilePointers {
        &self.file_pointers
    }

    /// Get the *fonts used* `foused01` chunk
    pub fn foused01(&self) -> &FontsUsed<'a> {
        &self.fonts
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

fn parse_header<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Header<'a>, E> {
    let (input, buf) = map(rest, Buf)(input)?;
    Ok((input, Header { buf }))
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
    let (i_foused01, _) = take(file_pointers.ofs_foused01)(data)?;
    let (_, fonts) =
        context("foused01", parse_chunk_head(CHUNK_FOUSED01, parse_foused01))(i_foused01)?;
    Ok((
        input,
        SDocV3 {
            header,
            file_pointers,
            fonts,
        },
    ))
}
