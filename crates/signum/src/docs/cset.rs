//! # (`cset`) The character set chunk

use bstr::BStr;
use nom::{bytes::complete::take, combinator::map, error::ParseError, multi::many0, IResult};

use crate::util::FourCC;

use super::Chunk;

/// Parse the `cset` chunk
pub fn parse_cset<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Vec<&'a BStr>, E> {
    many0(parse_cset_str)(input)
}

fn parse_cset_str<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], &'a BStr, E> {
    let (input, bytes) = take(10usize)(input)?;
    let bytes: &[u8] = bytes.splitn(2, |b| *b == 0).next().unwrap_or(bytes);
    Ok((input, BStr::new(bytes)))
}

/// # Character Sets (`cset`)
///
/// This chunks defines the mapping from a character set ID (0-9)
/// to the name of the character set. Ever is at most 10 characters
/// long and possibly nul-terminated.
///
/// The name corresponds to the font files in the `CHSETS` directory.
#[derive(Debug)]
pub struct CSet<'a> {
    /// The names of each file
    pub names: Vec<&'a BStr>,
}

impl<'a> CSet<'a> {
    /// Create a new character set
    pub fn new(names: Vec<&'a BStr>) -> Self {
        Self { names }
    }
}

impl<'a> Chunk<'a> for CSet<'a> {
    const TAG: FourCC = FourCC::_CSET;

    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>,
    {
        map(parse_cset, Self::new)(input)
    }
}
