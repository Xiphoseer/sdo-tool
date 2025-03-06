//! # Mapping charsets to unicode
use std::char::REPLACEMENT_CHARACTER;

use displaydoc::Display;
use nom::{
    bytes::complete::tag,
    character::complete::{hex_digit1, space1},
    combinator::{map_opt, map_res},
    error::ErrorKind,
    multi::many1,
    sequence::{preceded, tuple},
    Finish, IResult, Offset,
};
use smallvec::SmallVec;
use thiserror::Error;

mod atari;

pub use atari::{decode_atari, decode_atari_str};

/// Indicates that the implementing type encodes a mapping from a byte value to a sequence of unicode codepoints
pub trait ToUnicode {
    /// Decode a single byte
    fn decode(&self, cval: u8) -> &[char];
}

impl ToUnicode for [char; 128] {
    fn decode(&self, cval: u8) -> &[char] {
        std::slice::from_ref(&self[cval as usize])
    }
}

impl ToUnicode for [&[char]; 128] {
    fn decode(&self, cval: u8) -> &[char] {
        self[cval as usize]
    }
}

impl<T: ToUnicode> ToUnicode for Box<T> {
    fn decode(&self, cval: u8) -> &[char] {
        self.as_ref().decode(cval)
    }
}

impl<const N: usize> ToUnicode for [SmallVec<[char; N]>; 128] {
    fn decode(&self, cval: u8) -> &[char] {
        self[cval as usize].as_slice()
    }
}

impl ToUnicode for [[char; 2]; 128] {
    fn decode(&self, cval: u8) -> &[char] {
        match &self[cval as usize] {
            ['\0', '\0'] => &[],
            [c, '\0'] => std::slice::from_ref(c),
            slice => slice,
        }
    }
}

/// A mapping table for a charset
#[derive(Debug, Clone, PartialEq)]
pub enum Mapping {
    /// The corresponding unicode characters
    Single {
        /// characters in this variant
        chars: Box<[char; 128]>,
    },
    /// The corresponding unicode characters
    Static {
        /// characters in this variant
        chars: &'static [char; 128],
    },
    /// The corresponding unicode characters
    Dynamic {
        /// characters in this variant
        chars: Box<[SmallVec<[char; 2]>; 128]>,
    },
    /// The corresponding unicode characters
    BiLevel {
        /// characters in this variant
        chars: &'static [&'static [char]; 128],
    },
}

impl Mapping {
    /// Iterate over all mappings
    pub fn chars(&self) -> impl Iterator<Item = &[char]> {
        (0..128).map(move |c| self.decode(c))
    }

    /// Iterate over all mappings
    pub fn rows(&self) -> impl Iterator<Item = impl Iterator<Item = &[char]>> {
        (0..8).map(move |row| (0..16).map(move |c| self.decode((row << 4) + c)))
    }

    /// If the mapping can be encoded as a char array, return it
    pub fn as_char_array(&self) -> Option<&[char; 128]> {
        match self {
            Mapping::Single { chars } => Some(chars),
            Mapping::Static { chars } => Some(chars),
            Mapping::Dynamic { chars: _ } => None,
            Mapping::BiLevel { chars: _ } => None,
        }
    }
}

impl ToUnicode for Mapping {
    fn decode(&self, cval: u8) -> &[char] {
        match self {
            Mapping::Single { chars } => chars.decode(cval),
            Mapping::Static { chars } => chars.decode(cval),
            Mapping::Dynamic { chars } => chars.decode(cval),
            Mapping::BiLevel { chars } => chars.decode(cval),
        }
    }
}

impl Mapping {
    fn new(chars: [SmallVec<[char; 2]>; 128]) -> Self {
        if chars.iter().all(|c| c.len() <= 1) {
            let chars = chars.map(|v| v.first().copied().unwrap_or(REPLACEMENT_CHARACTER));
            Self::Single {
                chars: Box::new(chars),
            }
        } else {
            Self::Dynamic {
                chars: Box::new(chars),
            }
        }
    }
}

/// The mapping for ANTIKRO
pub const ANTIKRO_MAP: Mapping = Mapping::Static {
    chars: &antikro::MAP,
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

fn p_mapping_line(input: &str) -> IResult<&str, (u8, Vec<char>)> {
    tuple((
        hex_u8,
        many1(map_opt(preceded(space1, hex_u32), std::char::from_u32)),
    ))(input)
}

/// Parse a mapping file to a mapping struct
pub fn p_mapping_file(input: &str) -> Result<Mapping, MappingError> {
    const VEC: SmallVec<[char; 2]> = SmallVec::new_const();
    let mut chars = [VEC; 128];
    for (num, line) in input.lines().enumerate() {
        let valid = line.split('#').next().unwrap().trim();
        if !valid.is_empty() {
            let (_, (key, value)) = p_mapping_line(valid)
                .finish()
                .map_err(|e| MappingError::Problem(num, line.offset(e.input), e.code))?;
            if key > 127 {
                eprintln!("[signum.chsets.encoding] Invalid key {}, ignoring!", key);
            } else {
                chars[key as usize] = SmallVec::from_vec(value);
            }
        }
    }
    Ok(Mapping::new(chars))
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

    /// Decode a single ANTIKRO byte to a unicode char
    pub fn decode(byte: u8) -> char {
        match byte {
            0x00..=0x7f => MAP[byte as usize],
            _ => RCH,
        }
    }
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
