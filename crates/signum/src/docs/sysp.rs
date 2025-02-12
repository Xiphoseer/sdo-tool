//! # (`sysp`) The system parameters

use nom::{bytes::streaming::take, error::ParseError, number::complete::be_u16, IResult};
use serde::Serialize;

use crate::util::{Bytes16, Bytes32, FourCC};

use super::{bytes16, bytes32};

#[derive(Debug, Serialize)]
/// The system parameters chunk
pub struct SysP {
    /// Width of a space
    pub space_width: u16,
    /// ???
    pub letter_spacing: u16,
    /// (Vertical) distance between lines
    pub line_distance: u16,
    /// (Vertical) distance to index lines
    pub index_distance: u16,
    /// (Default) left page margin
    pub margin_left: u16,
    /// (Default) right page margin
    pub margin_right: u16,
    /// (Default) top page margin
    pub header: u16,
    /// (Default) bottom page margin
    pub footer: u16,
    /// Page length in editor units
    pub page_length: u16,
    /// Page numbering options
    pub page_numbering: PageNumbering,
    /// More layout options
    pub format_options: Bytes16,

    _opts_2: Bytes16,
    _opts_3: Bytes16,
    _opts_4: Bytes32,
}

/// Position of the page number
#[repr(u8)]
#[derive(Debug, Serialize, Copy, Clone, PartialEq, Eq)]
pub enum PageNumberPosition {
    #[doc(hidden)]
    _Invalid0 = 0,
    /// 0xC800 => 0b11001 => links
    Left = 1,
    /// 0xD000 => 0b11010 => mitte
    Middle = 2,
    /// 0xD800 => 0b11011 => rechts
    Right = 3,
    /// 0xE000 => 0b11100 => gerade
    Even = 4,
    /// 0xE800 => 0b11101 => ungerade
    Odd = 5,
    #[doc(hidden)]
    _Invalid6 = 6,
    #[doc(hidden)]
    _Invalid7 = 7,
}

impl PageNumberPosition {
    fn from_bits(input: u16) -> Self {
        unsafe { std::mem::transmute((input >> 11 & 0b111) as u8) }
    }
}

/// Whether the number is at the top or bottom of a page
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum PageNumberVertical {
    /// Numbering at the top
    Top,
    /// Numbering at the bottom
    Bottom,
}

impl PageNumberVertical {
    fn from_bits(value: u16) -> Self {
        match value & 0x4000 {
            0x4000 => Self::Top,
            _ => Self::Bottom,
        }
    }
}

/// Information on page numbering
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PageNumbering {
    enabled: bool,
    pos_y: PageNumberVertical,
    pos_x: PageNumberPosition,
    chset: u8,
}

/// Parse the `sysp` chunk
pub fn parse_sysp<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], SysP, E> {
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
            page_numbering: PageNumbering {
                enabled: page_numbering.0 & 0x8000 > 0,
                pos_y: PageNumberVertical::from_bits(page_numbering.0),
                pos_x: PageNumberPosition::from_bits(page_numbering.0),
                chset: (page_numbering.0 & 0b1111) as u8,
            },
            format_options,
            _opts_2: opts_2,
            _opts_3: opts_3,
            _opts_4: opts_4,
        },
    ))
}

impl<'a> super::Chunk<'a> for SysP {
    const TAG: crate::util::FourCC = FourCC::_SYSP;

    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>,
    {
        parse_sysp(input)
    }
}
