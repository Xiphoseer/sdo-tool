use bitflags::bitflags;
use nom::{
    bytes::complete::{tag, take, take_until, take_while},
    combinator::{map, map_res},
    error::ErrorKind,
    multi::{count, length_data, many0},
    number::complete::{be_u16, be_u32, be_u8},
    IResult,
};

use crate::{
    images::imc::{decode_imc, MonochromeScreen},
    util::{Bytes16, Bytes32},
    Buf,
};
use fmt::Debug;
use std::{borrow::Cow, fmt};

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
    pub left: u16,
    pub right: u16,
    pub top: u16,
    pub bottom: u16,
}

#[derive(Debug)]
pub struct Page {
    pub index: u16,
    pub phys_pnr: u16,
    pub log_pnr: u16,

    pub lines: (u8, u8),
    pub margin: Margin,
    pub numbpos: (u8, u8),
    pub kapitel: (u8, u8),
    pub intern: (u8, u8),
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
    pub cval: u8,
    pub cset: u8,
    pub offset: u16,
    pub style: Style,
}

/// The text buffer header
#[derive(Debug)]
pub struct TextBufferHeader {
    pub lines_total: u32,
}

pub fn parse_tebu_header(input: &[u8]) -> IResult<&[u8], TextBufferHeader> {
    let (input, lines_total) = be_u32(input)?;

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
pub struct LineIter<'a> {
    pub rest: &'a [u8],
}

impl<'a> Iterator for LineIter<'a> {
    type Item = Result<LineBuf<'a>, nom::Err<(&'a [u8], ErrorKind)>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.len() <= 4 {
            None
        } else {
            match parse_line_buf(self.rest) {
                Ok((rest, buf)) => {
                    self.rest = rest;
                    Some(Ok(buf))
                }
                Err(e) => Some(Err(e)),
            }
        }
    }
}

fn te(input: &[u8]) -> IResult<&[u8], Te> {
    let (input, cmd) = be_u16(input)?;

    // get the first sub-value
    let val = (cmd & 0x7E00) >> 9;
    let cset = ((cmd & 0x0180) >> 7) as u8;
    let cval = (cmd & 0x007F) as u8;

    // check whether the highest bit is set
    if cmd >= 0x8000 {
        Ok((
            input,
            Te {
                cval,
                cset,
                offset: val,
                style: Style::default(),
            },
        ))
    } else {
        let (input, extra) = be_u16(input)?;
        let offset = extra & 0x07ff;

        let underlined = val & 0x20 > 0;
        let footnote = val & 0x02 > 0;
        let cset = cset | (((val & 0x01) as u8) << 2);

        let sth1 = extra & 0x8000 > 0;
        let bold = extra & 0x4000 > 0;
        let italic = extra & 0x2000 > 0;
        let sth2 = extra & 0x1000 > 0;
        let small = extra & 0x0800 > 0;

        Ok((
            input,
            Te {
                cval,
                cset,
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
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LineBuf<'a> {
    pub skip: u16,
    pub data: &'a [u8],
}

bitflags! {
    pub struct Flags: u16 {
        const FLAG = 0x0001;
        const PNUM = 0x0080;
        /// Hauptzeile
        const LINE = 0x0400;
        /// Absatz
        const PARA = 0x0800;
        /// Kein-Text
        const ALIG = 0x1000;
        const PEND = 0x2000;
        const PNEW = 0x4000;
        const PAGE = 0x8000;
    }
}

#[derive(Debug)]
pub struct Line {
    pub flags: Flags,
    pub extra: u16,
    pub data: Vec<Te>,
}

pub fn parse_line(input: &[u8]) -> IResult<&[u8], Line> {
    let (input, bits) = bytes16(input)?;
    let flags = Flags::from_bits(bits.0) //
        .ok_or_else(|| anyhow::anyhow!("Unknown flags {:?}", bits))
        .unwrap();

    if flags.contains(Flags::PAGE) {
        let (input, pnum) = if flags.contains(Flags::PNUM) {
            be_u16(input)?
        } else {
            (input, 0)
        };
        Ok((
            input,
            Line {
                flags,
                extra: pnum,
                data: vec![],
            },
        ))
    } else {
        let (input, extra) = if flags.contains(Flags::FLAG) {
            be_u16(input)?
        } else {
            (input, 0)
        };
        let (input, text) = many0(te)(input)?;
        Ok((
            input,
            Line {
                flags,
                extra,
                data: text,
            },
        ))
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

#[derive(Debug)]
pub struct HCIMHeader {
    pub header_length: u32,
    pub img_count: u16,
    pub site_count: u16,
    pub c: Bytes32,
    pub d: Bytes32,
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct ImageSite {
    pub page: u16,
    pub pos_x: u16,
    pub pos_y: u16,
    pub _3: u16,
    pub _4: u16,
    pub _5: u16,
    pub sel_x: u16,
    pub sel_y: u16,
    pub sel_w: u16,
    pub sel_h: u16,
    pub _A: u16,
    pub _B: u16,
    pub _C: u16,
    pub img: u16,
    pub _E: u16,
    pub _F: Bytes16,
}

#[derive(Debug)]
pub struct HCIM<'a> {
    pub header: HCIMHeader,
    pub sites: Vec<ImageSite>,
    pub images: Vec<Buf<'a>>,
}

pub fn parse_image_buf(input: &[u8]) -> IResult<&[u8], Buf> {
    let (input, length2) = be_u32(input)?;
    let (input, buf2) = take((length2 - 4) as usize)(input)?;
    Ok((input, Buf(buf2)))
}

#[derive(Debug)]
pub struct Image<'a> {
    pub key: Cow<'a, str>,
    pub bytes: Buf<'a>,
    pub image: MonochromeScreen,
}

const ZERO: &[u8] = &[0];

pub fn parse_image(input: &[u8]) -> IResult<&[u8], Image> {
    let (input, key_bytes) = take_until(ZERO)(input)?;
    let key = String::from_utf8_lossy(key_bytes);

    let (input, _) = tag(ZERO)(input)?;
    let (input, bytes) = take(27usize - key_bytes.len())(input)?;
    let (input, image) = decode_imc(input)?;

    let bytes = Buf(bytes);
    Ok((input, Image { key, bytes, image }))
}

pub fn parse_hcim_header(input: &[u8]) -> IResult<&[u8], HCIMHeader> {
    let (input, header_length) = be_u32(input)?;
    let (input, img_count) = be_u16(input)?;
    let (input, site_count) = be_u16(input)?;
    let (input, c) = bytes32(input)?;
    let (input, d) = bytes32(input)?;

    Ok((
        input,
        HCIMHeader {
            header_length,
            img_count,
            site_count,
            c,
            d,
        },
    ))
}

#[allow(non_snake_case, clippy::just_underscores_and_digits)]
pub fn parse_hcim_img_ref(input: &[u8]) -> IResult<&[u8], ImageSite> {
    let (input, page) = be_u16(input)?;
    let (input, pos_x) = be_u16(input)?;
    let (input, pos_y) = be_u16(input)?;
    let (input, _3) = be_u16(input)?;
    let (input, _4) = be_u16(input)?;
    let (input, _5) = be_u16(input)?;
    let (input, sel_x) = be_u16(input)?;
    let (input, sel_y) = be_u16(input)?;
    let (input, sel_w) = be_u16(input)?;
    let (input, sel_h) = be_u16(input)?;
    let (input, _A) = be_u16(input)?;
    let (input, _B) = be_u16(input)?;
    let (input, _C) = be_u16(input)?;
    let (input, img) = be_u16(input)?;
    let (input, _E) = be_u16(input)?;
    let (input, _F) = bytes16(input)?;
    Ok((
        input,
        ImageSite {
            page,
            pos_x,
            pos_y,
            _3,
            _4,
            _5,
            sel_x,
            sel_y,
            sel_w,
            sel_h,
            _A,
            _B,
            _C,
            img,
            _E,
            _F,
        },
    ))
}

pub fn parse_hcim(input: &[u8]) -> IResult<&[u8], HCIM> {
    let (input, header) = parse_hcim_header(input)?;
    let (input, buf) = take(header.header_length as usize)(input)?;
    let (_, sites) = count(parse_hcim_img_ref, header.site_count as usize)(buf)?;
    let (input, images) = count(parse_image_buf, header.img_count as usize)(input)?;

    Ok((
        input,
        HCIM {
            header,
            sites,
            images,
        },
    ))
}
