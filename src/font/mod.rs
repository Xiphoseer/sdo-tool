#![allow(dead_code)]
use thiserror::Error;

pub mod editor;
pub mod printer;

use std::{char::REPLACEMENT_CHARACTER, io};

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Failed IO")]
    Io(#[from] io::Error),
}

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

/// The ATARI-ST character encoding 0 row
pub const ATARI_CHAR_MAP_0: [char; 14] = [
    '\0',
    '\u{21e7}',
    '\u{21e9}',
    '\u{21e8}',
    '\u{21e6}',
    '\u{1fbbd}',
    '\u{1fbbe}',
    '\u{1fbbf}',
    '\u{2713}',
    '\u{1F552}',
    '\u{1F514}',
    '\u{266A}',
    '\u{240C}',
    '\n',
];

/// The ATARI-ST character encoding 1 row
pub const ATARI_CHAR_MAP_1: [char; 12] = [
    '0', //'\u{1FBF0}',
    '1', //'\u{1FBF1}',
    '2', //'\u{1FBF2}',
    '3', //'\u{1FBF3}',
    '4', //'\u{1FBF4}',
    '5', //'\u{1FBF5}',
    '6', //'\u{1FBF6}',
    '7', //'\u{1FBF7}',
    '8', //'\u{1FBF8}',
    '9', //'\u{1FBF9}',
    '\u{0259}', '\u{241B}',
];

/// The ATARI-ST character encoding upper half
pub const ATARI_CHAR_MAP_UPPER: [char; 128] = [
    '\u{00C7}', '\u{00FC}', '\u{00E9}', '\u{00E2}', //
    '\u{00E4}', '\u{00E0}', '\u{00E5}', '\u{00E7}', //
    '\u{00EA}', '\u{00EB}', '\u{00E8}', '\u{00EF}', //
    '\u{00EE}', '\u{00EC}', '\u{00C4}', '\u{00C5}', //
    '\u{00C9}', '\u{00E6}', '\u{00C6}', '\u{00F4}', //
    '\u{00F6}', '\u{00F2}', '\u{00FB}', '\u{00F9}', //
    '\u{00FF}', '\u{00D6}', '\u{00DC}', '\u{00A2}', //
    '\u{00A3}', '\u{00A5}', '\u{00DF}', '\u{0192}', //
    '\u{00E1}', '\u{00ED}', '\u{00F3}', '\u{00FA}', //
    '\u{00F1}', '\u{00D1}', '\u{00AA}', '\u{00BA}', //
    '\u{00BF}', '\u{2310}', '\u{00AC}', '\u{00BD}', //
    '\u{00BC}', '\u{00A1}', '\u{00AB}', '\u{00BB}', //
    '\u{00E3}', '\u{00F5}', '\u{00D8}', '\u{00F8}', //
    '\u{0153}', '\u{0152}', '\u{00C0}', '\u{00C3}', //
    '\u{00D5}', '\u{00A8}', '\u{00B4}', '\u{2020}', //
    '\u{00B6}', '\u{00A9}', '\u{00AE}', '\u{2122}', //
    '\u{0133}', '\u{0132}', '\u{05D0}', '\u{05D1}', //
    '\u{05D2}', '\u{05D3}', '\u{05D4}', '\u{05D5}', //
    '\u{05D6}', '\u{05D7}', '\u{05D8}', '\u{05D9}', //
    '\u{05DB}', '\u{05DC}', '\u{05DE}', '\u{05E0}', //
    '\u{05E1}', '\u{05E2}', '\u{05E4}', '\u{05E6}', //
    '\u{05E7}', '\u{05E8}', '\u{05E9}', '\u{05EA}', //
    '\u{05DF}', '\u{05DA}', '\u{05DD}', '\u{05E3}', //
    '\u{05E5}', '\u{00A7}', '\u{2227}', '\u{221E}', //
    '\u{03B1}', '\u{03B2}', '\u{0393}', '\u{03C0}', //
    '\u{03A3}', '\u{03C3}', '\u{00B5}', '\u{03C4}', //
    '\u{03A6}', '\u{0398}', '\u{03A9}', '\u{03B4}', //
    '\u{222E}', '\u{03D5}', '\u{2208}', '\u{2229}', //
    '\u{2261}', '\u{00B1}', '\u{2265}', '\u{2264}', //
    '\u{2320}', '\u{2321}', '\u{00F7}', '\u{2248}', //
    '\u{00B0}', '\u{2022}', '\u{00B7}', '\u{221A}', //
    '\u{207F}', '\u{00B2}', '\u{00B3}', '\u{00AF}', //
];

pub fn decode_atari(byte: u8) -> char {
    match byte {
        0..=13 => ATARI_CHAR_MAP_0[byte as usize],
        16..=27 => ATARI_CHAR_MAP_1[(byte - 16) as usize],
        32..=126 => byte as char,
        127 => '\u{2302}',
        128..=255 => ATARI_CHAR_MAP_UPPER[(byte - 128) as usize],
        _ => REPLACEMENT_CHARACTER,
    }
}

/// The ANTIKRO Signum! font
pub mod antikro {
    #![allow(dead_code)]
    use super::LEGACY_7SEG_DIGITS as S;
    use std::char::REPLACEMENT_CHARACTER as RCH;

    pub const TIC: char = '\'';
    pub const NUL: char = '\0';
    pub const LFD: char = '\n';

    /// Private use characters for missing chars
    pub const A: (char, char, char, char, char, char, char) = (
        '\u{E003}', '\u{E004}', '\u{E005}', '\u{E006}', '\u{E008}', '\u{E00C}', '\u{E00E}',
    );

    /// Private use characters for missing chars
    pub const B: (char, char, char) = ('\u{E01D}', '\u{E01E}', '\u{E01F}');

    /// The ANTIKRO encoding row 0
    #[rustfmt::skip]
    pub const MAP: [char; 128] = [
        NUL, '{', '}', A.0, A.1, A.2, A.3, '↓', A.4, '←', '⏎', '→', A.5, '↑', A.6, '[',
        ']', '<', '>', S.0, S.1, S.2, S.3, S.4, S.5, S.6, S.7, S.8, S.9, B.0, B.1, B.2,
        '§', '!', '"', '#', '$', '%', '&', TIC, '(', ')', '*', '+', ',', '−', '.', '/',
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '«', '=', '»', '?',
        'ü', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
        'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'ö', 'Ü', 'ä', '^', '_',
        '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
        'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'Ö', '|', 'Ä', '-', 'ß',
    ];

    #[rustfmt::skip]
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

    #[rustfmt::skip]
    pub const SKIP: [u8; 128] = [
        0,   6,   6,   0,   0,   0,   0,   6,   0,   9,   0,   9,   0,   6,   0,   6,
        6,   6,   6,   9,   9,   9,   9,   9,   9,   9,   9,   9,   9,   0,   0,   0,
        4,   6,   6,   8,   5,   6,   6,   6,   6,   6,   9,   9,  16,  12,  16,   6,
        6,   6,   6,   6,   6,   6,   6,   6,   6,   6,  11,  11,  11,   9,  11,   6,
        7,   6,   6,   6,   6,   6,   6,   6,   6,   6,   6,   6,   6,   6,   6,   6,
        6,   6,   6,   6,   6,   6,   6,   6,   6,   6,   6,   7,   3,   7,   6,  12,
        6,  10,   6,  10,   6,  10,   6,  10,   6,   7,   7,   6,   6,  10,  10,  10,
       10,  10,  10,  10,   8,  10,  10,  10,  10,  10,  10,   4,   6,   4,  12,   6,
    ];

    pub fn decode(byte: u8) -> char {
        match byte {
            0x00..=0x7f => MAP[byte as usize],
            _ => RCH,
        }
    }
}

pub mod pris_11 {
    #![allow(dead_code)]
    use std::char::REPLACEMENT_CHARACTER as RCH;

    pub const TIC: char = '\'';
    pub const NUL: char = '\0';

    #[rustfmt::skip]
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

    #[rustfmt::skip]
    pub const SKIP: [u8; 128] = [
        0,   6,   6,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   6,
        6,   6,   6,   1,   8,   2,   0,   0,   0,   0,   0,   0,   0,   0,   0,   7,
        6,   6,   6,   6,   6,   6,   6,   6,   6,   6,   9,   9,  15,  12,  15,   6,
        6,   6,   6,   6,   6,   6,   6,   6,   6,   6,  10,  12,  10,  10,  10,   6,
        7,   6,   6,   6,   6,   6,   6,   6,   6,   6,   6,   6,   6,   6,   6,   6,
        6,   6,   6,   6,   6,   6,   6,   6,   6,   6,   6,   7,   6,   7,   6,  11,
        6,  10,   6,  10,   6,  10,   6,  10,   6,   7,   7,   6,   6,  10,  10,  10,
       10,  10,  10,  10,   8,  10,  10,  10,  10,  10,  10,   6,   6,   6,  12,   6,
    ];

    pub fn decode(byte: u8) -> char {
        match byte {
            0x00..=0x7f => MAP[byte as usize],
            _ => RCH,
        }
    }
}
