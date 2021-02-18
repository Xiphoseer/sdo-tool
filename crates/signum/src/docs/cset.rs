//! # (`cset`) The character set chunk

use std::borrow::Cow;

use nom::{bytes::complete::take, error::ParseError, multi::many0, IResult};

/// Parse the `cset` chunk
pub fn parse_cset<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Vec<Cow<'a, str>>, E> {
    many0(parse_cset_str)(input)
}

fn parse_cset_str<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Cow<'a, str>, E> {
    let (input, bytes) = take(10usize)(input)?;
    let bytes: &[u8] = bytes.splitn(2, |b| *b == 0).next().unwrap_or(bytes);
    Ok((input, String::from_utf8_lossy(bytes)))
}
