use bstr::BStr;
use nom::{
    bytes::complete::tag,
    combinator::map,
    error::{context, ContextError, ParseError},
    multi::length_value,
    number::complete::be_u16,
    sequence::{preceded, terminated},
    IResult, Parser,
};

use crate::util::Cow;

use super::Buf;

/// Parse a chunk
pub fn map_buf<'a, F, E: ParseError<&'a [u8]>>(
    parser: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Buf<'a>, E>
where
    F: Parser<&'a [u8], &'a [u8], E>,
{
    map(parser, Buf)
}

/// Parse a chunk
pub fn map_cow<'a, F, E: ParseError<&'a [u8]>>(
    parser: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Cow<'a>, E>
where
    F: Parser<&'a [u8], &'a [u8], E>,
{
    map(parser, Cow::new)
}

/// Parse a chunk
pub fn map_cow_owned<'a, 'b, F, E: ParseError<&'b [u8]>>(
    parser: F,
) -> impl FnMut(&'b [u8]) -> IResult<&'b [u8], Cow<'a>, E>
where
    F: Parser<&'b [u8], Vec<u8>, E>,
{
    map(parser, Cow::new_owned)
}

/// Parse a chunk
pub fn map_cow_to_owned<'a, 'b, F, E: ParseError<&'b [u8]>>(
    parser: F,
) -> impl FnMut(&'b [u8]) -> IResult<&'b [u8], Cow<'a>, E>
where
    F: Parser<&'b [u8], &'b [u8], E>,
{
    map_cow_owned(map(parser, Vec::from))
}

/// Parse a chunk
#[allow(dead_code)]
pub fn map_bstr<'a, F, E: ParseError<&'a [u8]>>(
    parser: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], &'a BStr, E>
where
    F: Parser<&'a [u8], &'a [u8], E>,
{
    map(parser, BStr::new)
}

/// Parse a Signum!3/4 style chunk header
fn parse_v3_chunk_head<'a, T, F, E: ParseError<&'a [u8]>>(
    chunk_tag: &'static [u8; 12],
    parser: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], T, E>
where
    F: Parser<&'a [u8], T, E>,
{
    let p = length_value(terminated(be_u16, be_u16), parser);
    preceded(tag(chunk_tag), p)
}

/// A Signum 3/4 type chunk
pub trait V3Chunk<'a>: Sized {
    /// Context to be set when parsing
    const CONTEXT: &'static str;
    /// The tag to check at the start
    const TAG: &'static [u8; 12];

    /// Parsing the content of a chunk
    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>;

    /// Parse the chunk with tag and length
    fn parse_chunk<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ContextError<&'a [u8]>,
        E: ParseError<&'a [u8]>,
    {
        context(Self::CONTEXT, parse_v3_chunk_head(Self::TAG, Self::parse))(input)
    }
}
