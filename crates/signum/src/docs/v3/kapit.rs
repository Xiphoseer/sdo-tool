use nom::{
    bytes::complete::take,
    combinator::rest,
    error::ParseError,
    number::complete::{be_i16, be_u16, be_u32},
    IResult,
};

use crate::util::{map_buf, Buf, V3Chunk};

/// Tag for *chapter*
pub const CHUNK_KAPIT01: &[u8; 12] = b"\0\0kapit 01\0\0";

/// Chunk *chapter* header `kapit 01`
#[allow(dead_code)]
#[derive(Debug)]
pub struct ChapterHeader<'a> {
    v1: u16,
    len1: u32,
    v3: u16,
    v4: [i16; 4],
    v8: i16,
    pub(super) v9: i16,
    pub(super) v10: i16,
    buf1: Buf<'a>, // len 66
    buf: Buf<'a>,  // size: len1
}

impl<'a> V3Chunk<'a> for ChapterHeader<'a> {
    const CONTEXT: &'static str = "kapit 01";

    const TAG: &'static [u8; 12] = CHUNK_KAPIT01;

    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>,
    {
        let (input, v1) = be_u16(input)?;
        let (input, len1) = be_u32(input)?;
        let (input, v3) = be_u16(input)?;
        let (input, v4) = be_i16(input)?;
        let (input, v5) = be_i16(input)?;
        let (input, v6) = be_i16(input)?;
        let (input, v7) = be_i16(input)?;
        let (input, v8) = be_i16(input)?;
        let (input, v9) = be_i16(input)?;
        let (input, v10) = be_i16(input)?;
        let v4 = [v4, v5, v6, v7];
        let (input, buf1) = map_buf(take(66usize))(input)?;
        let (input, buf) = map_buf(rest)(input)?;

        Ok((
            input,
            ChapterHeader {
                v1,
                len1,
                v3,
                v4,
                v8,
                v9,
                v10,
                buf1,
                buf,
            },
        ))
    }
}
