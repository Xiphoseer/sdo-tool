//! # (`sysp`) The system parameters

use nom::{bytes::streaming::take, number::complete::be_u16, IResult};

use crate::util::{Bytes16, Bytes32};

use super::{bytes16, bytes32};

#[derive(Debug)]
/// The system parameters chunk
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

/// Parse the `sysp` chunk
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
