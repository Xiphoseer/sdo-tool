//! # The outer container / structure

use nom::{
    bytes::complete::tag,
    combinator::eof,
    error::ParseError,
    multi::{length_data, many_till},
    number::complete::be_u32,
    sequence::preceded,
    IResult,
};

use crate::util::{Buf, FourCC};

use super::four_cc;
#[derive(Debug)]
/// One chunk in the document container
pub struct Chunk<'a> {
    /// The tag of the chunk
    pub tag: FourCC,
    /// The content of the chunk
    pub buf: Buf<'a>,
}

impl<'a> Chunk<'a> {
    /// Create a new chunk
    pub fn new(tag: FourCC, data: &'a [u8]) -> Self {
        Self {
            tag,
            buf: Buf(data),
        }
    }
}

/// A Signum! document container
#[derive(Debug)]
pub struct SDocContainer<'a> {
    /// The chunks in this container
    pub chunks: Vec<Chunk<'a>>,
}

/// Parse a single document chunk (i.e. tag + body)
pub fn parse_chunk<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Chunk<'a>, E> {
    let (rest, tag) = four_cc(input)?;
    let (rest, data) = length_data(be_u32)(rest)?;
    let chunk = Chunk::new(tag, data);
    Ok((rest, chunk))
}

/// Parse a Signum! document
pub fn parse_sdoc0001_container<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], SDocContainer<'a>, E> {
    let (input, (chunks, _)) = preceded(tag(b"sdoc"), many_till(parse_chunk, eof))(input)?;
    Ok((input, SDocContainer { chunks }))
}
