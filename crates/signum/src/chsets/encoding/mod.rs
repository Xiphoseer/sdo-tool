//! # Mapping charsets to unicode
use displaydoc::Display;
use nom::{
    bytes::complete::tag,
    character::complete::{hex_digit1, space1},
    combinator::map_res,
    error::ErrorKind,
    sequence::{preceded, tuple},
    Finish, IResult, Offset,
};
use thiserror::Error;

use std::char::REPLACEMENT_CHARACTER;

mod atari;

pub use atari::{decode_atari, decode_atari_str};

/// A mapping table for a charset
#[derive(Debug, Clone, PartialEq)]
pub struct Mapping {
    /// The corresponding unicode characters
    pub chars: [char; 128],
}

impl Mapping {
    /// Decodes a single character value
    pub fn decode(&self, cval: u8) -> char {
        self.chars[cval as usize]
    }
}

/// The mapping for ANTIKRO
pub const ANTIKRO_MAP: Mapping = Mapping {
    chars: antikro::MAP,
};

impl Default for &'_ Mapping {
    fn default() -> Self {
        &ANTIKRO_MAP
    }
}

/// Error when parsing a mapping
#[derive(Debug, Display, Error)]
pub enum MappingError {
    /// This is not implemented
    Unimplemented,
    /// Failed to parse ({2:?} at {0}:{1})
    Problem(usize, usize, ErrorKind),
}

fn hex_u8(input: &str) -> IResult<&str, u8> {
    preceded(
        tag("0x"),
        map_res(hex_digit1, |src| u8::from_str_radix(src, 16)),
    )(input)
}

fn hex_u32(input: &str) -> IResult<&str, u32> {
    preceded(
        tag("0x"),
        map_res(hex_digit1, |src| u32::from_str_radix(src, 16)),
    )(input)
}

fn p_mapping_line(input: &str) -> IResult<&str, (u8, u32)> {
    tuple((hex_u8, preceded(space1, hex_u32)))(input)
}

/// Parse a mapping file to a mapping struct
pub fn p_mapping_file(input: &str) -> Result<Mapping, MappingError> {
    let mut chars = [REPLACEMENT_CHARACTER; 128];
    for (num, line) in input.lines().enumerate() {
        let valid = line.split('#').next().unwrap().trim();
        if !valid.is_empty() {
            let (_, (key, value)) = p_mapping_line(valid)
                .finish()
                .map_err(|e| MappingError::Problem(num, line.offset(e.input), e.code))?;
            if key > 127 {
                eprintln!("[signum.chsets.encoding] Invalid key {}, ignoring!", key);
            } else if let Some(chr) = std::char::from_u32(value) {
                chars[key as usize] = chr;
            }
        }
    }
    Ok(Mapping { chars })
}

/// The unicode characters for legacy computing 7-segment digits 0 through 9
pub const LEGACY_7SEG_DIGITS: (char, char, char, char, char, char, char, char, char, char) = (
    '\u{1FBF0}',
    '\u{1FBF1}',
    '\u{1FBF2}',
    '\u{1FBF3}',
    '\u{1FBF4}',
    '\u{1FBF5}',
    '\u{1FBF6}',
    '\u{1FBF7}',
    '\u{1FBF8}',
    '\u{1FBF9}',
);

/// The ANTIKRO Signum! font
pub mod antikro {
    #![allow(dead_code)]
    use super::LEGACY_7SEG_DIGITS as S;
    use std::char::REPLACEMENT_CHARACTER as RCH;

    const TIC: char = '\'';
    const NUL: char = '\0';
    const LFD: char = '\n';

    /// Private use characters for missing chars
    const A: (char, char, char, char, char, char, char, char) = (
        '\u{E003}', '\u{E004}', '\u{E005}', '\u{E006}', '\u{E008}', '\u{E00A}', '\u{E00C}',
        '\u{E00E}',
    );

    /// Private use characters for missing chars
    const B: (char, char, char) = ('\u{E01D}', '\u{E01E}', '\u{E01F}');

    /// The ANTIKRO encoding
    #[rustfmt::skip]
    pub const MAP: [char; 128] = [
        NUL, '{', '}', A.0, A.1, A.2, A.3, '↓', A.4, '←', A.5, '→', A.6, '↑', A.7, '[',
        ']', '<', '>', S.0, S.1, S.2, S.3, S.4, S.5, S.6, S.7, S.8, S.9, B.0, B.1, B.2,
        '§', '!', '"', '#', '$', '%', '&', TIC, '(', ')', '*', '+', ',', '−', '.', '/',
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '«', '=', '»', '?',
        'ü', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
        'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'ö', 'Ü', 'ä', '^', '_',
        '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
        'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'Ö', '|', 'Ä', '-', 'ß',
    ];

    #[rustfmt::skip]
    /// The width of the characters in the ANTIKRO editor charset file
    pub const WIDTH: [u8; 128] = [
        0,   5,   5,   0,   0,   0,   0,   8,   0,  11,   0,  11,   0,   8,   0,   6,
        6,   8,   8,   6,   5,   6,   6,   6,   6,   6,   6,   6,   6,   0,   0,   7,
        8,   4,   6,   7,   9,   8,   9,   3,   5,   5,   7,   7,   4,   7,   4,   8,
        8,   5,   7,   7,   8,   7,   7,   7,   8,   7,   4,   4,   7,   7,   7,   7,
        8,  10,   9,  10,  10,   9,   8,  10,  11,   5,   6,  10,   8,  13,  10,  11,
        8,  11,   9,   8,   9,  11,  10,  14,  11,  10,   9,   8,  11,   7,   9,   9,
        4,   7,   8,   7,   8,   7,   6,   8,   8,   4,   4,   8,   5,  12,   8,   8,
        8,   8,   6,   7,   6,   8,   7,  11,   8,   7,   7,  11,   3,  10,   7,   8,
    ];

    /// Decode a single ANTIKRO byte to a unicode char
    pub fn decode(byte: u8) -> char {
        match byte {
            0x00..=0x7f => MAP[byte as usize],
            _ => RCH,
        }
    }
}

/// The `NORMANDE` standard font
pub mod normande {
    /// The widths of `NORMANDE` glyphs
    pub const WIDTH: [u8; 128] = [
        0,   5,   5,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   6,
        6,   8,   8,   7,   5,   7,   7,   6,   7,   7,   6,   7,   7,   0,   0,   0,
        8,   5,   8,   7,  10,   9,   9,   4,   7,   7,   7,   7,   5,   7,   5,   9,
        9,   6,   9,   9,   8,   8,   9,   8,   9,   9,   5,   5,   6,   7,   6,   7,
       10,  11,  13,  12,  13,  12,  12,  13,  13,   8,  11,  14,  12,  14,  10,  12,
       13,  12,  14,  10,  12,  11,  10,  15,  12,  12,  11,  10,  11,  10,   7,   9,
        4,  10,  10,  10,  10,  10,   8,  10,  10,   6,   7,  10,   6,  14,  10,  10,
       10,  10,   9,   8,   7,  10,   9,  14,  10,   9,   9,  12,   4,  11,   7,  10,
    ];
}

/// The `GROTMIKR` standard font
pub mod grotmikr {
    /// The widths of `GROTMIKR` glyphs
    pub const WIDTH: [u8; 128] = [
        0,   3,   3,   4,   4,  14,  13,  15,   0,   0,   0,   0,   0,   0,   0,   3,
        3,   4,   4,   4,   3,   4,   4,   4,   4,   4,   4,   4,   4,   0,   0,  15,
        4,   2,   3,   4,   4,   4,   5,   2,   3,   3,   4,   4,   2,   4,   2,   4,
        4,   3,   4,   4,   4,   4,   4,   4,   4,   4,   2,   2,   3,   4,   3,   4,
        4,   5,   4,   5,   5,   4,   4,   5,   5,   2,   3,   5,   4,   6,   5,   5,
        4,   5,   4,   4,   4,   5,   5,   9,   6,   5,   5,   4,   5,   4,   4,   5,
        2,   4,   4,   4,   4,   4,   3,   4,   4,   2,   2,   4,   2,   6,   4,   4,
        4,   4,   3,   4,   3,   4,   4,   6,   4,   4,   4,   5,   2,   5,   4,   4,
    ];
}

/// The `PINSEL` standard font
pub mod pinsel {
    /// The widths of `PINSEL` glyphs
    pub const WIDTH: [u8; 128] = [
        0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        0,   0,   0,   7,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        9,   5,   8,   7,  14,   9,   9,   5,   6,   6,   7,   7,   5,   7,   5,   9,
        9,   7,   8,   8,   9,   8,   8,   9,   8,   8,   5,   5,   6,   7,   6,   7,
        8,  11,  12,  12,  13,   9,  15,  12,  15,  12,  12,  11,  10,  15,  14,  12,
       11,  12,  13,  14,  12,  12,  14,  15,  15,  10,  15,   7,  12,   8,   8,   8,
        5,   8,   7,   6,   7,   7,   7,   8,   7,   5,   6,   8,   6,  11,   7,   7,
        9,   8,   6,   6,   6,   8,   8,  11,   7,   8,   7,  12,   4,  11,   7,   8,
    ];
}

/// The PRIS_11 charset
pub mod pris_11 {
    #![allow(dead_code)]
    use std::char::REPLACEMENT_CHARACTER as RCH;

    const TIC: char = '\'';
    const NUL: char = '\0';

    #[rustfmt::skip]
    /// The map of numbers to unicode characters
    pub const MAP: [char; 128] = [
        NUL, '{', '}', RCH, RCH, RCH, RCH, RCH, RCH, RCH, '⏎', RCH, RCH, RCH, RCH, '[',
        ']', '⟨', '⟩', '▒', '📧', '🤠', RCH, RCH, ' ', RCH, RCH, RCH, RCH, RCH, RCH, '☎',
        '§', '!', '"', '#', '$', '%', '&', TIC, '(', ')', '*', '+', ',', '−', '.', '/',
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?',
        'ü', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
        'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'ö', 'Ü', 'ä', '^', '_',
        '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
        'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'Ö', '|', 'Ä', '-', 'ß',
    ];

    #[rustfmt::skip]
    /// The width of the individual character
    pub const WIDTH: [u8; 128] = [
        0,   8,   8,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   8,
        8,   8,   8,  14,  13,  15,   0,   0,  10,   0,   0,   0,   0,   0,   0,  15,
        8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,
        8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,
        8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,
        8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   9,
        8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,
        8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,   8,
    ];

    /// Decode a single character
    pub fn decode(byte: u8) -> char {
        match byte {
            0x00..=0x7f => MAP[byte as usize],
            _ => RCH,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    #[test]
    fn test_decode() {
        assert_eq!(
            Cow::Borrowed("ANTIKRO"),
            super::decode_atari_str(b"ANTIKRO")
        );

        assert_eq!(
            super::decode_atari_str(b"S\x9ATT"),
            Cow::<'static, str>::Owned("SÜTT".to_string())
        )
    }
}
