//! # (`pbuf`) The page buffer

use nom::{
    bytes::{complete::tag, streaming::take},
    error::ParseError,
    number::complete::{be_u16, be_u32},
    IResult,
};

use crate::util::{Buf, Bytes16};

use super::bytes16;

#[derive(Debug)]
/// The page buffer
pub struct PBuf<'a> {
    /// The total number of pages
    pub page_count: u32,
    /// The length of the entry for each page
    pub kl: u32,
    /// The logical number for the first page
    pub first_page_nr: u32,
    /// A sparse map of pages, ordered by their index
    pub pages: Vec<Option<(Page, Buf<'a>)>>,
}

#[derive(Debug)]
/// The margins of a page
pub struct Margin {
    /// Width of the left margin
    pub left: u16,
    /// Distance of the right text margin from the left edge of the paper
    pub right: u16,
    /// Height of the header
    pub top: u16,
    /// Height of the footer
    pub bottom: u16,
}

#[derive(Debug)]
/// Structure representing a single page
pub struct Page {
    /// Page number over all documents from the same collection
    pub phys_pnr: u16,
    /// Page number within the current file / document
    pub log_pnr: u16,

    /// The total number of lines (?)
    pub lines: u16,
    /// The margins of this page
    pub margin: Margin,
    /// Specifies the position of the page number
    pub numbpos: Bytes16,
    /// May specify the current chapter (?)
    pub kapitel: Bytes16,
    /// Unknown
    pub intern: Bytes16,
}

fn parse_margin<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Margin, E> {
    let (input, left) = be_u16(input)?;
    let (input, right) = be_u16(input)?;
    let (input, top) = be_u16(input)?;
    let (input, bottom) = be_u16(input)?;

    Ok((
        input,
        Margin {
            left,
            right,
            top,
            bottom,
        },
    ))
}

fn parse_page<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], (Page, Buf<'a>), E> {
    let (input, phys_pnr) = be_u16(input)?;
    let (input, log_pnr) = be_u16(input)?;

    let (input, lines) = be_u16(input)?;
    let (input, margin) = parse_margin(input)?;
    let (input, numbpos) = bytes16(input)?;
    let (input, kapitel) = bytes16(input)?;
    let (input, intern) = bytes16(input)?;

    let (input, rest) = take(12usize)(input)?;
    Ok((
        input,
        (
            Page {
                phys_pnr,
                log_pnr,

                lines,

                margin,
                numbpos,
                kapitel,
                intern,
            },
            Buf(rest),
        ),
    ))
}

/// Parse a `pbuf` chunk
pub fn parse_pbuf<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], PBuf<'a>, E> {
    let (input, page_count) = be_u32(input)?;
    let (input, kl) = be_u32(input)?;
    let (input, first_page_nr) = be_u32(input)?;
    let (input, _) = tag(b"unde")(input)?;
    let (input, _) = tag(b"unde")(input)?;
    let (input, _) = tag(b"unde")(input)?;
    let (input, _) = tag(b"unde")(input)?;
    let (input, _) = tag(b"unde")(input)?;

    let mut pages = Vec::with_capacity(page_count as usize);
    let mut rest = input;

    for _ in 0..page_count {
        let (input, index) = be_u16(rest)?;
        let (input, (page, buf)) = parse_page(input)?;
        rest = input;
        let uindex = index as usize;
        if let Some(entry) = pages.get_mut(uindex) {
            *entry = Some((page, buf))
        } else {
            while pages.len() < uindex {
                pages.push(None);
            }
            pages.push(Some((page, buf)));
        }
    }

    Ok((
        rest,
        PBuf {
            page_count,
            kl,
            first_page_nr,
            pages,
        },
    ))
}
