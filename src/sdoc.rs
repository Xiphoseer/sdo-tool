use nom::{
    bytes::complete::{tag, take, take_while},
    combinator::{map, map_res},
    error::ErrorKind,
    multi::{count, length_data, many0},
    number::complete::{be_u16, be_u32, be_u8},
    IResult,
};

use crate::{
    font::antikro,
    util::{Bytes16, Bytes32},
    Buf,
};
use std::borrow::Cow;

/// A Signum! document container
#[derive(Debug)]
pub struct SDocContainer<'a> {
    pub parts: Vec<(&'a str, Buf<'a>)>,
}

fn take4<'a>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    take(4usize)(input)
}

/// Parse a Signum! document
pub fn parse_sdoc0001_container<'a>(input: &'a [u8]) -> IResult<&'a [u8], SDocContainer<'a>> {
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

#[derive(Debug)]
struct SDoc<'a> {
    charsets: Vec<Cow<'a, str>>,
}

fn parse_cset_str<'a>(input: &'a [u8]) -> IResult<&'a [u8], Cow<'a, str>> {
    let (input, bytes) = take_while(|b| b > 0)(input)?;
    let (input, _) = take(10 - bytes.len())(input)?;
    Ok((input, String::from_utf8_lossy(bytes)))
}

pub fn parse_cset<'a>(input: &'a [u8]) -> IResult<&'a [u8], Vec<Cow<'a, str>>> {
    many0(parse_cset_str)(input)
}

#[derive(Debug)]
pub struct SysP {
    space_width: u16,
    letter_spacing: u16,
    line_distance: u16,
    index_distance: u16,
    margin_left: u16,
    margin_right: u16,
    header: u16,
    footer: u16,
    page_length: u16,
    page_numbering: Bytes16,
    format_options: Bytes16,
    opts_2: Bytes16,
    opts_3: Bytes16,
    opts_4: Bytes32,
}

pub fn bytes16(input: &[u8]) -> IResult<&[u8], Bytes16> {
    map(be_u16, Bytes16)(input)
}

pub fn bytes32(input: &[u8]) -> IResult<&[u8], Bytes32> {
    map(be_u32, Bytes32)(input)
}

/*pub fn buffer(count: usize) -> impl FnMut(&[u8]) -> IResult<&[u8], Buf> {
    move |input: &[u8]| {
        map(take(count), Buf)(input)
    }
}*/

pub fn parse_sysp(input: &[u8]) -> IResult<&[u8], SysP> {
    let (input, _) = take(0x50usize)(input)?;

    // Standartseitenformat
    let (input, space_width) = be_u16(input)?; // Leerzeichenbreite
    let (input, letter_spacing) = be_u16(input)?; // Sperrung
    let (input, line_distance) = be_u16(input)?; // Hauptzeilenabstand
    let (input, index_distance) = be_u16(input)?; // Indexabstand
    let (input, margin_left) = be_u16(input)?; // Linker Rand (0)
    let (input, margin_right) = be_u16(input)?; // Rechter Rand (6.5 * 90)
    let (input, header) = be_u16(input)?; // Kopfzeilen (0.1 * 54)
    let (input, footer) = be_u16(input)?; // Fußzeilen (0.1 * 54)
    let (input, page_length) = be_u16(input)?; // Seitenlänge (10.4 * 54)
    let (input, page_numbering) = bytes16(input)?; // H5800 == keine Seitennummerierung
    let (input, format_options) = bytes16(input)?; // X10011 == format. optionen
    let (input, opts_2) = bytes16(input)?; // H302 == trennen
    let (input, opts_3) = bytes16(input)?; // 0 == randausgleiche und Sperren
    let (input, opts_4) = bytes32(input)?; // 1 == nicht einrücken, absatzabstand mitkorrigieren

    Ok((
        input,
        SysP {
            space_width,
            letter_spacing,
            line_distance,
            index_distance,
            margin_left,
            margin_right,
            header,
            footer,
            page_length,
            page_numbering,
            format_options,
            opts_2,
            opts_3,
            opts_4,
        },
    ))
}

#[derive(Debug)]
pub struct PBuf<'a> {
    pub page_count: u32,
    pub kl: u32,
    pub first_page_nr: u32,
    pub vec: Vec<(Page, Buf<'a>)>,
}

#[derive(Debug)]
pub struct Margin {
    left: u16,
    right: u16,
    top: u16,
    bottom: u16,
}

#[derive(Debug)]
pub struct Page {
    index: u16,
    phys_pnr: u16,
    log_pnr: u16,

    lines: (u8, u8),
    margin: Margin,
    numbpos: (u8, u8),
    kapitel: (u8, u8),
    intern: (u8, u8),
}

fn parse_margin(input: &[u8]) -> IResult<&[u8], Margin> {
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

fn be_2_u8(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
    let (input, v1) = be_u8(input)?;
    let (input, v2) = be_u8(input)?;
    Ok((input, (v1, v2)))
}

fn parse_page(input: &[u8]) -> IResult<&[u8], (Page, Buf)> {
    let (input, index) = be_u16(input)?;
    let (input, phys_pnr) = be_u16(input)?;
    let (input, log_pnr) = be_u16(input)?;

    let (input, lines) = be_2_u8(input)?;
    let (input, margin) = parse_margin(input)?;
    let (input, numbpos) = be_2_u8(input)?;
    let (input, kapitel) = be_2_u8(input)?;
    let (input, intern) = be_2_u8(input)?;

    let (input, rest) = take(12usize)(input)?;
    Ok((
        input,
        (
            Page {
                index,
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

pub fn parse_pbuf(input: &[u8]) -> IResult<&[u8], PBuf> {
    let (input, page_count) = be_u32(input)?;
    let (input, kl) = be_u32(input)?;
    let (input, first_page_nr) = be_u32(input)?;
    let (input, _) = tag(b"unde")(input)?;
    let (input, _) = tag(b"unde")(input)?;
    let (input, _) = tag(b"unde")(input)?;
    let (input, _) = tag(b"unde")(input)?;
    let (input, _) = tag(b"unde")(input)?;

    let (input, vec) = count(parse_page, page_count as usize)(input)?;
    //let (input, d) = be_i32(input)?;
    Ok((
        input,
        PBuf {
            page_count,
            kl,
            first_page_nr,
            vec,
        },
    ))
}

#[derive(Debug, Copy, Clone)]
pub struct Style {
    pub underlined: bool,
    pub footnote: bool,
    pub sth1: bool,
    pub bold: bool,
    pub italic: bool,
    pub sth2: bool,
    pub small: bool,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            underlined: false,
            footnote: false,
            sth1: false,
            bold: false,
            italic: false,
            sth2: false,
            small: false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
/// A single text character
pub struct Te {
    pub char: char,
    pub width: u8,
    pub offset: u16,
    pub style: Style,
}

/// The text buffer header
#[derive(Debug)]
pub struct TextBufferHeader {
    pub lines_total: u32,
    //pub a: Bytes16,
    //pub b: u16,
    //pub c: Bytes16,
}

pub fn parse_tebu_header(input: &[u8]) -> IResult<&[u8], TextBufferHeader> {
    let (input, lines_total) = be_u32(input)?;
    //let (input, a) = bytes16(input)?;
    //let (input, b) = be_u16(input)?;
    //let (input, c) = bytes16(input)?;

    Ok((
        input,
        TextBufferHeader {
            lines_total, //a, b, c
        },
    ))
}

/// The text buffer
#[derive(Debug)]
pub struct TeBu<'a> {
    pub header: TextBufferHeader,
    pub lines: Vec<LineBuf<'a>>,
}

#[derive(Clone)]
pub struct LineIter<'a>(pub &'a [u8]);

impl<'a> Iterator for LineIter<'a> {
    type Item = Result<LineBuf<'a>, nom::Err<(&'a [u8], ErrorKind)>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            None
        } else {
            match parse_line_buf(self.0) {
                Ok((rest, buf)) => {
                    self.0 = rest;
                    Some(Ok(buf))
                }
                Err(e) => Some(Err(e)),
            }
        }
    }
}

fn te<F: Fn(u8) -> char>(decode: F) -> impl Fn(&[u8]) -> IResult<&[u8], Te> {
    move |input: &[u8]| {
        let (input, cmd) = be_u16(input)?;

        // get the first sub-value
        let val = (cmd & 0x7E00) >> 9;
        let _cset = (cmd & 0x0180) >> 7;
        let chr = cmd & 0x007F;

        let chru = chr as usize;
        let width = crate::font::antikro::WIDTH[chru];
        let _skip = crate::font::antikro::SKIP[chru];

        // check whether the highest bit is set
        if cmd >= 0x8000 {
            Ok((
                input,
                Te {
                    char: decode(chr as u8),
                    width,
                    offset: val,
                    style: Style::default(),
                },
            ))
        } else {
            let (input, extra) = be_u16(input)?;
            let offset = extra & 0x07ff;

            let underlined = val & 0x20 > 0;
            let footnote = val & 0x02 > 0;
            let sth1 = extra & 0x8000 > 0;
            let bold = extra & 0x4000 > 0;
            let italic = extra & 0x2000 > 0;
            let sth2 = extra & 0x1000 > 0;
            let small = extra & 0x0800 > 0;

            Ok((
                input,
                Te {
                    char: decode(chr as u8),
                    width,
                    offset,
                    style: Style {
                        underlined,
                        footnote,
                        sth1,
                        bold,
                        italic,
                        sth2,
                        small,
                    },
                },
            ))

            /*let _style = (val & 0x30) >> 4;
            let kind = val & 0x0f;
            match kind {
                0 => {


                    let _mod = (extra & 0xf000) >> 12;

                }
                2 => {
                    if chr > 0 {
                        let (input, _offset) = be_u16(input)?;
                        Ok((input, Te::Break(chr)))
                    } else {
                        Ok((input, Te::Break(chr)))
                    }
                }
                6 => Ok((input, Te::Paragraph(chr))),
                _ => Ok((input, Te::Unknown(cmd))),
            }*/
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LineBuf<'a> {
    pub skip: u16,
    pub data: &'a [u8],
}

#[derive(Debug)]
pub enum Line {
    Zero(Vec<Te>),
    Paragraph(Vec<Te>),
    Paragraph1(Bytes16, Vec<Te>),
    Line(Vec<Te>),
    Line1(Bytes16, Vec<Te>),
    Heading(Vec<Te>),
    Some(Vec<Te>),
    Heading2(Vec<Te>),
    FirstPageEnd,
    NewPage(u16),
    PageEnd(u16),
    Unknown(Bytes16),
}

pub fn parse_line(input: &[u8]) -> IResult<&[u8], Line> {
    let (input, kind_tag) = bytes16(input)?;
    match kind_tag.0 {
        0x0000 => {
            let (input, text) = many0(te(antikro::decode))(input)?;
            Ok((input, Line::Zero(text)))
        }
        0x0C00 => {
            let (input, text) = many0(te(antikro::decode))(input)?;
            Ok((input, Line::Paragraph(text)))
        }
        0x0C01 => {
            let (input, unknown) = bytes16(input)?;
            let (input, text) = many0(te(antikro::decode))(input)?;
            Ok((input, Line::Paragraph1(unknown, text)))
        }
        0x0400 => {
            let (input, text) = many0(te(antikro::decode))(input)?;
            Ok((input, Line::Line(text)))
        }
        0x0401 => {
            let (input, unknown) = bytes16(input)?;
            let (input, text) = many0(te(antikro::decode))(input)?;
            Ok((input, Line::Line1(unknown, text)))
        }
        0x1000 => {
            let (input, text) = many0(te(antikro::decode))(input)?;
            Ok((input, Line::Heading(text)))
        }
        0x1400 => {
            let (input, text) = many0(te(antikro::decode))(input)?;
            Ok((input, Line::Some(text)))
        }
        0x1C00 => {
            let (input, text) = many0(te(antikro::decode))(input)?;
            Ok((input, Line::Heading2(text)))
        }

        0xA000 => Ok((input, Line::FirstPageEnd)),
        0xA080 => {
            let (input, page_num) = be_u16(input)?;
            Ok((input, Line::PageEnd(page_num)))
        }
        0xC080 => {
            let (input, page_num) = be_u16(input)?;
            Ok((input, Line::NewPage(page_num)))
        }
        _ => Ok((input, Line::Unknown(kind_tag))),
    }
}

impl<'a> LineBuf<'a> {
    pub fn _parse(self) -> Result<Line, nom::Err<(&'a [u8], ErrorKind)>> {
        parse_line(self.data).map(|(_, line)| line)
    }
}

fn parse_line_buf(input: &[u8]) -> IResult<&[u8], LineBuf> {
    let (input, skip) = be_u16(input)?;
    let (input, data) = length_data(be_u16)(input)?;
    Ok((input, LineBuf { skip, data }))
}

#[allow(clippy::many_single_char_names)]
pub fn _parse_tebu(input: &[u8]) -> IResult<&[u8], TeBu> {
    let (input, header) = parse_tebu_header(input)?;
    let (input, lines) = many0(parse_line_buf)(input)?;

    Ok((input, TeBu { header, lines }))
}
