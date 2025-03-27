use std::borrow::Cow;

use nom::{
    bytes::complete::{take, take_while},
    combinator::{map, map_parser},
    error::ParseError,
    multi::many0,
    IResult,
};

use crate::{chsets::encoding::decode_atari_str, util::V3Chunk};

/// Tag for *fonts used*
pub const CHUNK_FOUSED01: &[u8; 12] = b"\0\0foused01\0\0";

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

    fn new(fonts: Vec<FontUsed<'a>>) -> Self {
        Self { fonts }
    }
}

fn parse_font_used<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], FontUsed<'a>, E> {
    let (input, index) = nom::number::complete::u8(input)?;
    let (input, name) = map_parser(take(9usize), take_while(|c| c != 0u8))(input)?;
    Ok((input, (index, decode_atari_str(name))))
}

impl<'a> V3Chunk<'a> for FontsUsed<'a> {
    const CONTEXT: &'static str = "foused01";

    const TAG: &'static [u8; 12] = CHUNK_FOUSED01;

    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>,
    {
        map(many0(parse_font_used), FontsUsed::new)(input)
    }
}
