//! # (`tebu`) The text buffer
use bitflags::bitflags;
use bstr::ByteSlice;
use log::info;
use nom::{
    combinator::{iterator, map_parser, map_res, verify},
    error::{context, ContextError, ErrorKind, ParseError},
    multi::{length_data, many0},
    number::complete::{be_u16, be_u32},
    sequence::tuple,
    IResult,
};
use serde::Serialize;

use crate::{
    chsets::{
        cache::{ChsetCache, DocumentFontCacheInfo},
        encoding::{ToUnicode, ANTIKRO_MAP},
        UseMatrix,
    },
    util::FourCC,
};

use super::bytes16;

bitflags! {
    /// The font modifiers applied to a character
    #[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Serialize)]
    pub struct Style : u16 {
        /// Whether the char is underlined
        const UNDERLINED = 0x20;
        /// unknown 1
        const V = 0x10;
        /// unknown 1
        const W = 0x08;
        /// unknown 1
        const X = 0x04;
        /// Whether the char is a footnote
        const FOOTNOTE = 0x02;

        /// Whether the character is double width
        const WIDE = 0x8000;
        /// Whether the char is bold
        const BOLD = 0x4000;
        /// Whether the char is italic
        const ITALIC = 0x2000;
        /// Whether the character is tall
        const TALL = 0x1000;
        /// Whether the char is small
        const SMALL = 0x0800;
    }
}

impl Style {
    /// Check whether the character is underlined
    pub fn is_underlined(&self) -> bool {
        self.contains(Self::UNDERLINED)
    }

    /// Check whether the character is double width
    pub fn is_wide(&self) -> bool {
        self.contains(Self::WIDE)
    }

    /// Check whether the character is bold
    pub fn is_bold(&self) -> bool {
        self.contains(Self::BOLD)
    }

    /// Check whether the character is italic
    pub fn is_italic(&self) -> bool {
        self.contains(Self::ITALIC)
    }

    /// Check whether the character is tall
    pub fn is_tall(&self) -> bool {
        self.contains(Self::TALL)
    }

    /// Check whether the character is small
    pub fn is_small(&self) -> bool {
        self.contains(Self::SMALL)
    }

    /// Check whether the character is small
    pub fn is_footnote(&self) -> bool {
        self.contains(Self::FOOTNOTE)
    }
}

#[derive(Debug, Copy, Clone, Serialize)]
/// A single text character
pub struct Char {
    /// The number of the character
    pub cval: u8,
    /// The number of the character set
    pub cset: u8,
    /// The horizontal offset from the previous position
    pub offset: u16,
    /// The style of the character
    pub style: Style,
}

/// The text buffer header
#[derive(Debug, Default, Serialize)]
pub struct TextBufferHeader {
    /// The total number of lines in the buffer
    pub lines_total: u32,
}

/// Parse a `tebu` chunk header
pub fn parse_tebu_header<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], TextBufferHeader, E> {
    let (input, lines_total) = be_u32(input)?;

    Ok((
        input,
        TextBufferHeader {
            lines_total, //a, b, c
        },
    ))
}

/// The text buffer
#[derive(Debug, Default, Serialize)]
pub struct TeBu {
    /// The header of the buffer
    pub header: TextBufferHeader,
    /// The indicidual lines
    ///
    /// A line is a sequence of characters in the same vertical position
    pub pages: Vec<PageText>,
    /// Unknown
    pub trailer: u32,
}

impl TeBu {
    /// Return a [UseMatrix] for all pages in this text buffer chunk
    pub fn use_matrix(&self) -> UseMatrix {
        UseMatrix::from(&self.pages[..])
    }
}

#[derive(Clone)]
/// An iterator over characters in a line
pub struct LineIter<'a> {
    rest: &'a [u8],
}

impl<'a> LineIter<'a> {
    /// Return the underlying slice for the rest of the line
    pub fn as_slice(&self) -> &'a [u8] {
        self.rest
    }
}

impl<'a> Iterator for LineIter<'a> {
    type Item = Result<LineBuf<'a>, nom::Err<nom::error::Error<&'a [u8]>>>;
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

fn te<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Char, E> {
    let (input, cmd) = be_u16(input)?;

    // get the first sub-value
    let val = (cmd & 0x7E00) >> 9;
    let cset = ((cmd & 0x0180) >> 7) as u8;
    let cval = (cmd & 0x007F) as u8;

    // check whether the highest bit is set
    if cmd >= 0x8000 {
        Ok((
            input,
            Char {
                cval,
                cset,
                offset: val,
                style: Style::default(),
            },
        ))
    } else {
        let (input, extra) = be_u16(input)?;
        let offset = extra & 0x07ff;

        let cset = cset | (((val & 0x01) as u8) << 2);
        let style = Style::from_bits_retain(extra & 0xF800 | val);
        let char = Char {
            cval,
            cset,
            offset,
            style,
        };
        Ok((input, char))
    }
}

#[derive(Debug, Copy, Clone)]
/// A single line
pub struct LineBuf<'a> {
    /// The vertical offset that should be skipped
    pub skip: u16,
    /// The data of the line
    pub data: &'a [u8],
}

bitflags! {
    /// The flags that of a line
    #[derive(Serialize, Debug)]
    #[serde(transparent)]
    pub struct Flags: u16 {
        /// ???
        const FLAG = 0x0001;
        /// ???
        const F1 = 0x0002;
        /// ???
        const F2 = 0x0004;
        /// ???
        const F3 = 0x0008;
        /// ???
        const F5 = 0x0010;
        /// ???
        const F6 = 0x0020;
        /// ???
        const F7 = 0x0040;
        /// This is followed by an associated page number
        const PNUM = 0x0080;
        /// ???
        const F8 = 0x0100;
        /// ???
        const F9 = 0x0200;
        /// Hauptzeile
        const LINE = 0x0400;
        /// Absatz
        const PARA = 0x0800;
        /// Kein-Text
        const ALIG = 0x1000;
        /// This is the end of a page
        const PEND = 0x2000;
        /// This is the start of a page
        const PNEW = 0x4000;
        /// This is a page ctrl line
        const PAGE = 0x8000;
    }
}

#[derive(Debug, Serialize)]
/// Structure that holds a parsed line
pub struct Line {
    len: usize,
    /// The flags for the line
    pub flags: Flags,
    /// The extra value (usually page number)
    pub extra: u16,
    /// The characters in the line
    pub data: Vec<Char>,
}

impl Line {
    /// Iterator over all charactes (and positions) in a line
    pub fn characters(&self) -> LineCharIter<'_> {
        LineCharIter {
            x: 0,
            inner: self.data.iter(),
        }
    }

    /// If all characters in the line have the same style and charset, return it
    pub fn line_style(&self) -> Option<(Style, u8)> {
        let (first, rest) = self.data.split_first()?;
        let style = first.style;
        let cset = first.cset;
        if rest.iter().all(|k| k.style == style && k.cset == cset) {
            Some((style, cset))
        } else {
            None
        }
    }

    /// Get the decoded text on the line
    pub fn text(&self, fc: &ChsetCache, dfci: &DocumentFontCacheInfo) -> String {
        let mut text = String::new();
        let mut last_char_width = 0;
        for k in &self.data {
            let cset = dfci.cset(fc, k.cset);
            let mapping = cset.and_then(|c| c.map()).unwrap_or(&ANTIKRO_MAP);
            let decoded = mapping.decode(k.cval);

            let lcw = last_char_width.into();
            if k.offset >= lcw {
                let hdiff = k.offset - lcw;
                if hdiff > 2 && !text.is_empty() {
                    text.push(' ');
                }
            }

            let width = dfci.width(fc, k);
            last_char_width = width;
            for &chr in decoded {
                text.push(chr);
            }
        }
        text
    }
}

/// Iterator over the characters in a line, keeping track of horizontal position
pub struct LineCharIter<'a> {
    x: u16,
    inner: std::slice::Iter<'a, Char>,
}

impl<'a> Iterator for LineCharIter<'a> {
    type Item = (u16, &'a Char);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next()?;
        self.x += next.offset;
        Some((self.x, next))
    }
}

/// Parse a line from its buffer
pub fn parse_line<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Line, E> {
    let len = input.len();
    let (input, bits) = bytes16(input)?;
    let flags = Flags::from_bits(bits.0).expect("Unknown flags");

    if flags.contains(Flags::PAGE) {
        let (input, pnum) = if flags.contains(Flags::PNUM) {
            be_u16(input)?
        } else {
            (input, 0)
        };
        Ok((
            input,
            Line {
                len,
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
                len,
                flags,
                extra,
                data: text,
            },
        ))
    }
}

impl<'a> LineBuf<'a> {
    /// Parse the line contained in the buffer
    pub fn parse(self) -> Result<Line, nom::Err<nom::error::Error<&'a [u8]>>> {
        parse_line(self.data).map(|(_, line)| line)
    }
}

fn parse_line_buf<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], LineBuf<'a>, E> {
    let (input, skip) = be_u16(input)?;
    let (input, data) = length_data(be_u16)(input)?;
    Ok((input, LineBuf { skip, data }))
}

fn parse_buffered_line<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], (u16, Line), E> {
    tuple((
        be_u16,
        map_parser(length_data(verify(be_u16, |l| *l < 0x8000)), parse_line),
    ))(input)
}

fn parse_page_start_line<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], (u16, u16), E> {
    map_res::<_, _, _, nom::error::Error<&'a [u8]>, _, _, _>(parse_buffered_line, |(a, l)| {
        log::trace!("START [skip={}] {:?} [extra={}]", a, l.flags, l.extra);
        if l.flags.contains(Flags::PAGE & Flags::PNEW) {
            Ok((a, l.extra))
        } else {
            Err("Expected the start of a page!")
        }
    })(input)
    .map_err(|e| e.map(|e| E::from_error_kind(e.input, e.code)))
}

/// Holds the lines of a complete page
#[derive(Debug, Serialize)]
pub struct PageText {
    /// The index of the page
    pub index: u16,
    /// The horizontal offset of the start page marker
    pub skip: u16,
    /// The horizontal offset of the end page marker
    pub rskip: u16,
    /// The content
    pub content: Vec<(u16, Line)>,
}

/// Parse the text of an entire page
pub fn parse_page_text<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], PageText, E> {
    log::trace!("{:?} of {}", &input[..4], input.len());
    if input.len() == 4 {
        // no more pages
        return Err(nom::Err::Error(E::from_error_kind(
            input,
            ErrorKind::Verify,
        )));
    }
    let (input, (lskip, index)) = parse_page_start_line(input)?;
    let mut iter = iterator(input, parse_buffered_line);

    let mut content = vec![];
    while let Some((skip, line)) = (&mut iter).next() {
        log::trace!(
            "[skip=0x{:04x}] {:?} [extra={}, len={}]",
            skip,
            line.flags,
            line.extra,
            line.len,
        );
        if !line.flags.contains(Flags::PAGE) {
            content.push((skip, line));
            continue;
        }

        if !line.flags.contains(Flags::PEND) {
            log::warn!(
                "Broken text buffer, please send in this document for investigation. (len = {})",
                line.data.len()
            );
            break;
        }

        if line.extra != index {
            let (rest, ()) = iter.finish()?;
            panic!(
                "Broken text buffer: {} != {} [bytes remaining={}]",
                line.extra,
                index,
                rest.len()
            );
        }
        return iter.finish().map(|(rest, ())| {
            let text = PageText {
                index,
                skip: lskip,
                rskip: skip,
                content,
            };
            (rest, text)
        });
    }

    match iter.finish() {
        Ok((rest, ())) => {
            let diff = input.len() - rest.len();
            log::warn!(
                "Trying to recover: input={}, rest={}, diff={}",
                input.len(),
                rest.len(),
                diff
            );
            recover_page(lskip, index, content, input)
        }
        Err(e) => Err(e),
    }
}

fn recover_page<'a, E: ParseError<&'a [u8]>>(
    lskip: u16,
    index: u16,
    content: Vec<(u16, Line)>,
    rest: &'a [u8],
) -> IResult<&'a [u8], PageText, E> {
    // We left the loop without finding a PAGE END flag, try to recover
    let pnum = index.to_be_bytes();
    let ahead = rest.get(..6096).unwrap_or(rest);
    let offset = ahead.find([0xA0, 0x80, pnum[0], pnum[1]]);
    if let Some(offset) = offset {
        log::warn!("Invalid page, but found end marker at +{}", offset);
        let rest = &rest[(offset - 4)..];
        let (rest, (rskip, end_line)) = parse_buffered_line::<nom::error::Error<_>>(rest)
            .expect("expected page end line at marker");
        log::debug!("{:?}", end_line);
        Ok((
            rest,
            PageText {
                index,
                skip: lskip,
                rskip,
                content,
            },
        ))
    } else {
        Err(nom::Err::Failure(E::from_error_kind(rest, ErrorKind::Tag)))
    }
}

/// Parse a `tebu` chunk
pub fn parse_tebu<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], TeBu, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    let (mut input, header) = context("tebu_header", parse_tebu_header)(input)?;
    let mut pages = Vec::new();
    let mut it = iterator(input, context("page_text", parse_page_text));
    for i in &mut it {
        pages.push(i);
    }
    match it.finish() {
        Ok((rest, ())) => {
            let (rest, trailer) = be_u32(rest)?;
            input = rest;
            info!("Parsed {} pages", pages.len());
            let tebu = TeBu {
                header,
                pages,
                trailer,
            };
            Ok((input, tebu))
        }
        Err(e) => match e {
            nom::Err::Incomplete(needed) => panic!("Incomplete: {:?}", needed),
            nom::Err::Error(e) => Err(nom::Err::Error(e)),
            nom::Err::Failure(e) => Err(nom::Err::Failure(e)),
        },
    }
}

impl<'a> super::Chunk<'a> for TeBu {
    const TAG: crate::util::FourCC = FourCC::_TEBU;

    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: nom::error::ParseError<&'a [u8]> + nom::error::ContextError<&'a [u8]>,
    {
        context("tebu", parse_tebu)(input)
    }
}
