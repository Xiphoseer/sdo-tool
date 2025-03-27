use std::num::NonZeroU32;

use nom::{combinator::verify, error::ParseError, multi::many1, number::complete::be_u32, IResult};

use crate::util::V3Chunk;

/// Tag for *file pointers*
pub const CHUNK_FLPTRS01: &[u8; 12] = b"\0\0flptrs01\0\0";

/// `flptrs01` chunk
#[allow(dead_code)]
#[derive(Debug)]
pub struct FilePointers {
    /// Offset of `foused01`
    pub ofs_foused01: u32,
    /// Offset of `params01`
    pub ofs_params01: u32,
    /// Offset of `cdilist0`
    pub ofs_cdilist0: Option<NonZeroU32>,
    /// Offset of `foxlist `
    pub ofs_foxlist: Option<NonZeroU32>,
    u5: Option<NonZeroU32>,
    u6: Option<NonZeroU32>,
    u7: Option<NonZeroU32>,
    u8: Option<NonZeroU32>,
    /// Offsets of `kapit 01`
    pub ofs_chapters: Vec<u32>,
}

fn opt_be_nonzero_u32<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Option<NonZeroU32>, E> {
    let (input, val) = be_u32(input)?;
    Ok((input, NonZeroU32::new(val)))
}

impl<'a> V3Chunk<'a> for FilePointers {
    fn parse<E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], FilePointers, E> {
        let (input, ofs_foused01) = verify(be_u32, |v| *v > 0)(input)?;
        let (input, ofs_params01) = verify(be_u32, |v| *v > 0)(input)?;
        let (input, ofs_cdilist0) = opt_be_nonzero_u32(input)?;
        let (input, ofs_foxlist) = opt_be_nonzero_u32(input)?;
        let (input, u5) = opt_be_nonzero_u32(input)?;
        let (input, u6) = opt_be_nonzero_u32(input)?;
        let (input, u7) = opt_be_nonzero_u32(input)?;
        let (input, u8) = opt_be_nonzero_u32(input)?;
        let (input, ofs_chapters) = many1(be_u32)(input)?;
        Ok((
            input,
            FilePointers {
                ofs_foused01,
                ofs_params01,
                ofs_cdilist0,
                ofs_foxlist,
                u5,
                u6,
                u7,
                u8,
                ofs_chapters,
            },
        ))
    }

    const CONTEXT: &'static str = "flptrs01";

    const TAG: &'static [u8; 12] = CHUNK_FLPTRS01;
}
