//! # Signum 3/4 fonts
//! 
//! Signum 3/4 fonts consist of a sequence of chunks, each compressed (or encrypted)
//! with some as yet unknown mechanism.

use nom::{
    bytes::complete::tag,
    combinator::{map, map_opt, rest},
    error::{context, ContextError, ParseError},
    multi::length_value,
    number::complete::{be_u16, be_u32},
    sequence::{preceded, tuple},
    IResult,
};

use crate::util::{map_buf, Buf};

/// Tag (magic bytes) for Signum! 3/4 fonts
pub const TAG_CSET2: &[u8; 12] = b"\0\x02chset001\0\0";

/// The header of a `chset001` font file
#[allow(dead_code)]
#[derive(Debug)]
pub struct ChsetHeader<'a> {
    v1: u32,
    rest: Buf<'a>,
}

impl<'a> ChsetChunk<'a> for ChsetHeader<'a> {
    const TAG: &'static str = "chset001";

    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>,
    {
        let (input, v1) = be_u32(input)?;
        let (input, rest) = map_buf(rest)(input)?;
        Ok((input, ChsetHeader { v1, rest }))
    }
}

/// A `fdeskr01` font file chunk
#[allow(dead_code)]
#[derive(Debug)]
pub struct FontDescriptor<'a> {
    v1: u32,
    rest: Buf<'a>,
}

impl<'a> ChsetChunk<'a> for FontDescriptor<'a> {
    const TAG: &'static str = "fdeskr01";

    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>,
    {
        let (input, v1) = be_u32(input)?;
        let (input, rest) = map_buf(rest)(input)?;
        Ok((input, FontDescriptor { v1, rest }))
    }
}

/// A `lgtab001` font file chunk
#[allow(dead_code)]
#[derive(Debug)]
pub struct LigatureTable<'a> {
    v1: u32,
    rest: Buf<'a>,
}

impl<'a> ChsetChunk<'a> for LigatureTable<'a> {
    const TAG: &'static str = "lgtab001";

    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>,
    {
        let (input, v1) = be_u32(input)?;
        let (input, rest) = map_buf(rest)(input)?;
        Ok((input, LigatureTable { v1, rest }))
    }
}

/// A `chars001` font file chunk
#[allow(dead_code)]
#[derive(Debug)]
pub struct Characters<'a> {
    v1: u32,
    rest: Buf<'a>,
}

impl<'a> ChsetChunk<'a> for Characters<'a> {
    const TAG: &'static str = "chars001";

    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>,
    {
        let (input, v1) = be_u32(input)?;
        let (input, rest) = map_buf(rest)(input)?;
        Ok((input, Characters { v1, rest }))
    }
}

/// A `kerntab1` font file chunk
#[allow(dead_code)]
#[derive(Debug)]
pub struct KerningTable<'a> {
    rest: Buf<'a>,
}

impl<'a> ChsetChunk<'a> for KerningTable<'a> {
    const TAG: &'static str = "kerntab1";

    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>,
    {
        let (input, rest) = map_buf(rest)(input)?;
        Ok((input, KerningTable { rest }))
    }
}

/// A Signum 3/4 font
pub struct ChsetV2 {}

trait ChsetChunk<'a>: Sized {
    /// The tag to check at the start
    const TAG: &'static str;

    /// Parsing the content of a chunk
    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>;

    fn parse_chunk<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>,
        E: ContextError<&'a [u8]>,
    {
        let (input, rest) = context(
            Self::TAG,
            length_value(
                map_opt(
                    preceded(
                        tuple((tag(b"\0\x02"), tag(Self::TAG), tag(b"\0\0"))),
                        be_u16,
                    ),
                    |l| l.checked_sub(14),
                ),
                Self::parse,
            ),
        )(input)?;
        Ok((input, rest))
    }
}

/// Parse a Signum! document
pub fn parse_chset_v2<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ChsetV2, E>
where
    E: ParseError<&'a [u8]>,
    E: ContextError<&'a [u8]>,
{
    let _data = input;

    let (input, chset001) = ChsetHeader::parse_chunk(input)?;
    log::info!("{:#?}", chset001);
    let (input, fdeskr01) = FontDescriptor::parse_chunk(input)?;
    log::info!("{:#?}", fdeskr01);
    let (input, lgtab001) = LigatureTable::parse_chunk(input)?;
    log::info!("{:#?}", lgtab001);
    let (input, chars001) = Characters::parse_chunk(input)?;
    log::info!("{:#?}", chars001);
    let (input, kerntab1) = if !input.is_empty() {
        map(KerningTable::parse_chunk, Some)(input)?
    } else {
        (input, None)
    };
    if let Some(kerntab1) = kerntab1 {
        log::info!("{:#?}", kerntab1);
    }
    Ok((input, ChsetV2 {}))
}
