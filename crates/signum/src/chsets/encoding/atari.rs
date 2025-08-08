use std::{borrow::Cow, char::REPLACEMENT_CHARACTER, io};

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
    '\n',
    '\u{266A}',
    '\u{240C}',
    '\r',
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
    '\u{00C7}', '\u{00FC}', '\u{00E9}', '\u{00E2}', // 0x80
    '\u{00E4}', '\u{00E0}', '\u{00E5}', '\u{00E7}', // 0x84
    '\u{00EA}', '\u{00EB}', '\u{00E8}', '\u{00EF}', // 0x88
    '\u{00EE}', '\u{00EC}', '\u{00C4}', '\u{00C5}', // 0x8C
    '\u{00C9}', '\u{00E6}', '\u{00C6}', '\u{00F4}', // 0x90
    '\u{00F6}', '\u{00F2}', '\u{00FB}', '\u{00F9}', // 0x94
    '\u{00FF}', '\u{00D6}', '\u{00DC}', '\u{00A2}', // 0x98
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

/// Decode the ATARI char map to unicode
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

fn first_non_ascii_char(input: &[u8]) -> Option<usize> {
    input.iter().copied().position(|p| !(32..127).contains(&p))
}

/// Decode an ATARI-ST String into an UTF-8 String
pub fn decode_atari_str(input: &[u8]) -> Cow<'_, str> {
    if let Some(pos) = first_non_ascii_char(input) {
        let (first, rest) = input.split_at(pos);
        let start = unsafe { std::str::from_utf8_unchecked(first) };
        let mut string = start.to_owned();
        string.extend(rest.iter().copied().map(decode_atari));
        Cow::Owned(string)
    } else {
        Cow::Borrowed(unsafe { std::str::from_utf8_unchecked(input) })
    }
}

/// Decode an ATARI-ST String into an UTF-8 String
pub fn decode_atari_string(input: Vec<u8>) -> String {
    if let Some(pos) = first_non_ascii_char(&input) {
        let (first, rest) = input.split_at(pos);
        let start = unsafe { std::str::from_utf8_unchecked(first) };
        let mut string = start.to_owned();
        string.extend(rest.iter().copied().map(decode_atari));
        string
    } else {
        unsafe { String::from_utf8_unchecked(input) }
    }
}

/// An iterator over lines encoded in Atari ST character encoding
pub struct AtariStrLines<B> {
    buf: B,
}

impl<B: io::BufRead> AtariStrLines<B> {
    /// Create a new instance of this iterator
    pub fn new(buf: B) -> Self {
        AtariStrLines { buf }
    }
}

impl<B: io::BufRead> Iterator for AtariStrLines<B> {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<io::Result<String>> {
        let mut buf = Vec::new();
        match self.buf.read_until(b'\n', &mut buf) {
            Ok(0) => None,
            Ok(_n) => {
                if buf.ends_with(b"\n") {
                    buf.pop();
                    if buf.ends_with(b"\r") {
                        buf.pop();
                    }
                }
                Some(Ok(decode_atari_string(buf)))
            }
            Err(e) => Some(Err(e)),
        }
    }
}
