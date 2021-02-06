//! # (`cset`) The character set chunk

use std::borrow::Cow;

use nom::{
    bytes::{complete::take_while, streaming::take},
    multi::many0,
    IResult,
};

/// Parse the `cset` chunk
pub fn parse_cset(input: &[u8]) -> IResult<&[u8], Vec<Cow<str>>> {
    many0(parse_cset_str)(input)
}

fn parse_cset_str(input: &[u8]) -> IResult<&[u8], Cow<str>> {
    let (input, bytes) = take_while(|b| b > 0)(input)?;
    let (input, _) = take(10 - bytes.len())(input)?;
    Ok((input, String::from_utf8_lossy(bytes)))
}
