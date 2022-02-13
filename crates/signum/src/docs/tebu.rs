//! # (`tebu`) The text buffer
use bitflags::bitflags;
use nom::{
    combinator::{iterator, map_parser, map_res},
    error::ErrorKind,
    multi::{length_data, many0},
    number::complete::{be_u16, be_u32},
    sequence::tuple,
    IResult,
};

use super::bytes16;

#[derive(Debug, Copy, Clone, Default)]
/// The style of a character
pub struct Style {
    /// Whether the char is underlined
    pub underlined: bool,
    /// Whether the char is a footnote
    pub footnote: bool,
    /// Whether the character is double width
    pub wide: bool,
    /// Whether the char is bold
    pub bold: bool,
    /// Whether the char is italic
    pub italic: bool,
    /// Whether the character is tall
    pub tall: bool,
    /// Whether the char is small
    pub small: bool,
}

#[derive(Debug, Copy, Clone)]
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
#[derive(Debug)]
pub struct TextBufferHeader {
    /// The total number of lines in the buffer
    pub lines_total: u32,
}

/// Parse a `tebu` chunk header
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
    /// The header of the buffer
    pub header: TextBufferHeader,
    /// The indicidual lines
    ///
    /// A line is a sequence of characters in the same vertical position
    pub lines: Vec<LineBuf<'a>>,
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

fn te(input: &[u8]) -> IResult<&[u8], Char> {
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
            Char {
                cval,
                cset,
                offset,
                style: Style {
                    underlined,
                    footnote,
                    wide: sth1,
                    bold,
                    italic,
                    tall: sth2,
                    small,
                },
            },
        ))
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
    pub struct Flags: u16 {
        /// ???
        const FLAG = 0x0001;
        /// This is followed by an associated page number
        const PNUM = 0x0080;
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

#[derive(Debug)]
/// Structure that holds a parsed line
pub struct Line {
    /// The flags for the line
    pub flags: Flags,
    /// The extra value (usually page number)
    pub extra: u16,
    /// The characters in the line
    pub data: Vec<Char>,
}

/// Parse a line from its buffer
pub fn parse_line(input: &[u8]) -> IResult<&[u8], Line> {
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
    /// Parse the line contained in the buffer
    pub fn parse(self) -> Result<Line, nom::Err<nom::error::Error<&'a [u8]>>> {
        parse_line(self.data).map(|(_, line)| line)
    }
}

fn parse_line_buf(input: &[u8]) -> IResult<&[u8], LineBuf> {
    let (input, skip) = be_u16(input)?;
    let (input, data) = length_data(be_u16)(input)?;
    Ok((input, LineBuf { skip, data }))
}

fn parse_buffered_line(input: &[u8]) -> IResult<&[u8], (u16, Line)> {
    tuple((be_u16, map_parser(length_data(be_u16), parse_line)))(input)
}

fn parse_page_start_line(input: &[u8]) -> IResult<&[u8], (u16, u16)> {
    map_res(parse_buffered_line, |(a, l)| {
        if l.flags.contains(Flags::PAGE & Flags::PNEW) {
            Ok((a, l.extra))
        } else {
            Err("Expected the start of a page!")
        }
    })(input)
}

/// Holds the lines of a complete page
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
pub fn parse_page_text(input: &[u8]) -> IResult<&[u8], PageText> {
    let (input, (lskip, index)) = parse_page_start_line(input)?;
    let mut iter = iterator(input, parse_buffered_line);

    let mut content = vec![];
    for (skip, line) in &mut iter {
        if !line.flags.contains(Flags::PAGE) {
            content.push((skip, line));
            continue;
        }

        if !line.flags.contains(Flags::PEND) {
            panic!("This is an unknown case, please send in this document for investigation.")
        }

        assert_eq!(line.extra, index);
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
        Ok((rest, ())) => Err(nom::Err::Failure(nom::error::Error {
            input: rest,
            code: ErrorKind::Eof,
        })),
        Err(e) => Err(e),
    }
}

/// Parse a `tebu` chunk
pub fn parse_tebu(input: &[u8]) -> IResult<&[u8], TeBu> {
    let (input, header) = parse_tebu_header(input)?;
    let (input, lines) = many0(parse_line_buf)(input)?;

    Ok((input, TeBu { header, lines }))
}
