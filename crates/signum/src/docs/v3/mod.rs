//! # Signum! 3/4 Documents
//!
//! Signum! 3/4 uses a completely different format from 1/2.

use std::num::NonZeroU32;

use nom::{
    bytes::complete::tag,
    combinator::rest,
    error::ParseError,
    multi::length_value,
    number::complete::{be_u16, be_u32},
    sequence::{preceded, terminated},
    IResult, Parser,
};

use crate::util::Buf;

/// Tag for a v3 document
pub const TAG_SDOC3: &[u8; 12] = b"\0\0sdoc  03\0\0";

const CHUNK_FLPTRS01: &[u8; 12] = b"\0\0flptrs01\0\0";

/// Document root
#[allow(dead_code)]
#[derive(Debug)]
pub struct SDocV3<'a> {
    /// Header buffer
    pub header: Buf<'a>,
    /// The file pointers
    file_pointers: FilePointers,
}

/// `flptrs01` chunk
#[allow(dead_code)]
#[derive(Debug)]
struct FilePointers {
    pub ofs_foused01: Option<NonZeroU32>,
    pub ofs_params01: Option<NonZeroU32>,
    pub ofs_cdilist0: Option<NonZeroU32>,
    pub ofs_foxlist: Option<NonZeroU32>,
    u5: Option<NonZeroU32>,
    u6: Option<NonZeroU32>,
    u7: Option<NonZeroU32>,
    u8: Option<NonZeroU32>,
    pub ofs_content_first: Option<NonZeroU32>,
    pub ofs_content_last: Option<NonZeroU32>,
}

fn parse_header<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Buf<'a>, E> {
    let (input, rest) = rest(input)?;
    Ok((input, Buf(rest)))
}

fn be_u32_bounded<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Option<NonZeroU32>, E> {
    if input.is_empty() {
        Ok((input, None))
    } else {
        let (input, val) = be_u32(input)?;
        Ok((input, NonZeroU32::new(val)))
    }
}

fn parse_flptrs01<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], FilePointers, E> {
    let (input, ofs_foused01) = be_u32_bounded(input)?;
    let (input, ofs_params01) = be_u32_bounded(input)?;
    let (input, ofs_cdilist0) = be_u32_bounded(input)?;
    let (input, ofs_foxlist) = be_u32_bounded(input)?;
    let (input, u5) = be_u32_bounded(input)?;
    let (input, u6) = be_u32_bounded(input)?;
    let (input, u7) = be_u32_bounded(input)?;
    let (input, u8) = be_u32_bounded(input)?;
    let (input, ofs_content_first) = be_u32_bounded(input)?;
    let (input, ofs_content_last) = be_u32_bounded(input)?;
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
            ofs_content_first,
            ofs_content_last,
        },
    ))
}

/// Parse a chunk
pub fn parse_chunk_head<'a, T, F, E: ParseError<&'a [u8]>>(
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
pub fn parse_sdoc_v3<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], SDocV3<'a>, E> {
    let (input, header) = parse_chunk_head(TAG_SDOC3, parse_header)(input)?;
    let (input, file_pointers) = parse_chunk_head(CHUNK_FLPTRS01, parse_flptrs01)(input)?;
    Ok((
        input,
        SDocV3 {
            header,
            file_pointers,
        },
    ))
}
