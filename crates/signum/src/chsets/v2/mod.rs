//! # Signum 3/4 fonts
//!
//! Signum 3/4 fonts consist of a sequence of chunks, each compressed (or encrypted)
//! with some as yet unknown mechanism:
//!
//! - The compressed data starts with a 32-bit uncompressed length specifier
//! - The empty array is encoded as `0x1F 0xDA`
//! - The compressed length is always a multiple of 2
//! - A bitflip anywhere in the uncompressed chunk leads to changes all over the compressed chunk

use core::fmt;

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, map_opt, rest},
    error::{context, ContextError, ParseError},
    multi::length_value,
    number::complete::{be_u16, be_u32},
    sequence::{preceded, terminated, tuple},
    IResult,
};

use crate::util::{map_buf, Buf};

/// Tag (magic bytes) for Signum!3 (uncompressed) fonts
pub const TAG_CHSET: &[u8; 12] = b"\0\0chset001\0\0";

/// Tag (magic bytes) for Signum!4 (compressed) fonts
pub const TAG_CHSET_COMPRESSED: &[u8; 12] = b"\0\x02chset001\0\0";

/// The header of a `chset001` font file
#[allow(dead_code)]
#[derive(Debug)]
pub struct ChsetHeader<'a> {
    rest: Buf<'a>,
}

impl<'a> ChsetChunk<'a> for ChsetHeader<'a> {
    const TAG: &'static str = "chset001";

    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>,
    {
        let (input, rest) = map_buf(rest)(input)?;
        Ok((input, ChsetHeader { rest }))
    }
}

/// A `fdeskr01` font file chunk
#[allow(dead_code)]
#[derive(Debug)]
pub struct FontDescriptor<'a> {
    rest: Buf<'a>,
}

impl<'a> ChsetChunk<'a> for FontDescriptor<'a> {
    const TAG: &'static str = "fdeskr01";

    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>,
    {
        let (input, rest) = map_buf(rest)(input)?;
        Ok((input, FontDescriptor { rest }))
    }
}

/// A `lgtab001` font file chunk
#[allow(dead_code)]
#[derive(Debug)]
pub struct LigatureTable<'a> {
    rest: Buf<'a>,
}

impl<'a> ChsetChunk<'a> for LigatureTable<'a> {
    const TAG: &'static str = "lgtab001";

    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>,
    {
        let (input, rest) = map_buf(rest)(input)?;
        Ok((input, LigatureTable { rest }))
    }
}

/// A `chars001` font file chunk
#[allow(dead_code)]
#[derive(Debug)]
pub struct Characters<'a> {
    rest: Buf<'a>,
}

impl<'a> ChsetChunk<'a> for Characters<'a> {
    const TAG: &'static str = "chars001";

    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>,
    {
        let (input, rest) = map_buf(rest)(input)?;
        Ok((input, Characters { rest }))
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

fn parse_compressed<'a, E, T: ChsetChunk<'a>>(input: &'a [u8]) -> IResult<&'a [u8], Chunk<'a, T>, E>
where
    E: ParseError<&'a [u8]>,
{
    let (input, len) = be_u32(input)?;
    let (input, bytes) = map_buf(rest)(input)?;
    Ok((input, Chunk::Compressed { len, bytes }))
}

/// A Signum 3/4 font
pub struct ChsetV2<'a> {
    /// `chset001` chunk
    pub chset001: Chunk<'a, ChsetHeader<'a>>,
    /// `fdeskr01` chunk
    pub fdeskr01: Chunk<'a, FontDescriptor<'a>>,
    /// `lgtab001` chunk
    pub lgtab001: Chunk<'a, LigatureTable<'a>>,
    /// `chars001` chunk
    pub chars001: Chunk<'a, Characters<'a>>,
    /// `kerntab1` chunk
    pub kerntab1: Option<Chunk<'a, KerningTable<'a>>>,
}

/// A character set chunk
#[derive(Clone)]
pub enum Chunk<'a, T> {
    /// An uncompresssed (Signum!3) chunk
    Plain(T),
    /// A compressed (Signum!4) chunk
    Compressed {
        /// Uncompressed length
        len: u32,
        /// Compressed bytes
        bytes: Buf<'a>,
    },
}

impl<'a, T: ChsetChunk<'a>> fmt::Debug for Chunk<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plain(arg0) => arg0.fmt(f),
            Self::Compressed { len, bytes } => f
                .debug_struct("CompressedChunk")
                .field("tag", &T::TAG)
                .field("len", len)
                .field("bytes", bytes)
                .finish(),
        }
    }
}

trait ChsetChunk<'a>: fmt::Debug + Sized {
    /// The tag to check at the start
    const TAG: &'static str;

    /// Parsing the content of a chunk
    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>;

    fn parse_chunk<E>(input: &'a [u8]) -> IResult<&'a [u8], Chunk<'a, Self>, E>
    where
        E: ParseError<&'a [u8]>,
        E: ContextError<&'a [u8]>,
    {
        let (input, rest) = context(
            Self::TAG,
            alt((
                length_value(
                    map_opt(
                        preceded(tuple((tag(b"\0\x02"), tag(Self::TAG))), be_u32),
                        |l| l.checked_sub(14),
                    ),
                    parse_compressed,
                ),
                length_value(
                    terminated(
                        preceded(tuple((tag(b"\0\0"), tag(Self::TAG))), be_u32),
                        be_u16,
                    ),
                    map(Self::parse, Chunk::Plain),
                ),
            )),
        )(input)?;
        Ok((input, rest))
    }
}

/// Parse a Signum! 3/4 chset
pub fn parse_chset_v2<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], ChsetV2<'a>, E>
where
    E: ParseError<&'a [u8]>,
    E: ContextError<&'a [u8]>,
{
    let _data = input;

    let (input, chset001) = ChsetHeader::parse_chunk(input)?;
    let (input, fdeskr01) = FontDescriptor::parse_chunk(input)?;
    let (input, lgtab001) = LigatureTable::parse_chunk(input)?;
    let (input, chars001) = Characters::parse_chunk(input)?;
    let (input, kerntab1) = if !input.is_empty() {
        map(KerningTable::parse_chunk, Some)(input)?
    } else {
        (input, None)
    };
    Ok((
        input,
        ChsetV2 {
            chset001,
            fdeskr01,
            lgtab001,
            chars001,
            kerntab1,
        },
    ))
}
