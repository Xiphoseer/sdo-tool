use nom::{
    bytes::complete::{tag, take},
    combinator::map_res,
    multi::length_data,
    number::complete::be_u32,
    IResult,
};

use crate::util::Buf;

/// A Signum! document container
#[derive(Debug)]
pub struct SDocContainer<'a> {
    pub parts: Vec<(&'a str, Buf<'a>)>,
}

fn take4(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take(4usize)(input)
}

/// Parse a Signum! document
pub fn parse_sdoc0001_container(input: &[u8]) -> IResult<&[u8], SDocContainer> {
    let (input, _) = tag(b"sdoc")(input)?;
    let mut parts = Vec::new();
    let mut input = input;
    while !input.is_empty() {
        let (rest, key): (&[u8], &str) = map_res(take4, std::str::from_utf8)(input)?;
        let (rest, data) = length_data(be_u32)(rest)?;
        parts.push((key, Buf(data)));
        input = rest;
    }

    Ok((input, SDocContainer { parts }))
}
