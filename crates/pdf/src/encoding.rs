//! Text encodings

use std::{
    error::Error,
    fmt,
    io::{self, Write},
};

/// Encode the input slice so that it can be decoded with the
/// *Ascii85Decode* filter. Returns the number of written bytes.
pub fn ascii_85_encode<W: Write>(data: &[u8], w: &mut W) -> io::Result<usize> {
    let mut ctr = 0;
    let mut cut = 75;

    let mut chunks_exact = data.chunks_exact(4);
    for group in &mut chunks_exact {
        let buf = u32::from_be_bytes([group[0], group[1], group[2], group[3]]);
        if buf == 0 {
            w.write_all(&[0x7A])?; // `z`
            ctr += 1;
        } else {
            let (c_5, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let (c_4, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let (c_3, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let (c_2, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let c_1 = buf as u8 + 33;
            w.write_all(&[c_1, c_2, c_3, c_4, c_5])?;
            ctr += 5;
        }

        if ctr >= cut {
            w.write_all(&[0x0A])?;
            ctr += 1;
            cut = ctr + 75;
        }
    }
    match *chunks_exact.remainder() {
        [b_1] => {
            let buf = u32::from_be_bytes([b_1, 0, 0, 0]) / (85 * 85 * 85);
            let (c_2, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let c_1 = buf as u8 + 33;
            w.write_all(&[c_1, c_2, 0x7E, 0x3E])?;
            ctr += 4;
        }
        [b_1, b_2] => {
            let buf = u32::from_be_bytes([b_1, b_2, 0, 0]) / (85 * 85);
            let (c_3, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let (c_2, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let c_1 = buf as u8 + 33;
            w.write_all(&[c_1, c_2, c_3, 0x7E, 0x3E])?;
            ctr += 5;
        }
        [b_1, b_2, b_3] => {
            let buf = u32::from_be_bytes([b_1, b_2, b_3, 0]) / 85;
            let (c_4, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let (c_3, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let (c_2, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let c_1 = buf as u8 + 33;
            w.write_all(&[c_1, c_2, c_3, c_4, 0x7E, 0x3E])?;
            ctr += 6;
        }
        _ => {
            w.write_all(&[0x7E, 0x3E])?;
            ctr += 2;
        }
    }

    Ok(ctr)
}

#[derive(Debug)]
/// Codepoint U+{0:04x} is not valid in PDFDocEncoding
#[allow(clippy::upper_case_acronyms)]
pub struct PDFDocEncodingError(char);

impl Error for PDFDocEncodingError {}
impl fmt::Display for PDFDocEncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Codepoint U+{:04x} is not valid in PDFDocEncoding",
            self.0 as u32
        )
    }
}

fn pdf_char_encode(chr: char) -> Result<u8, PDFDocEncodingError> {
    match u32::from(chr) {
        0x00..=0x17 | 0x20..=0x7E | 0xA1..=0xff => Ok(chr as u8),

        0x02D8 => Ok(0x18),
        0x02C7 => Ok(0x19),
        0x02C6 => Ok(0x1A),
        0x02D9 => Ok(0x1B),
        0x02DD => Ok(0x1C),
        0x02DB => Ok(0x1D),
        0x02DA => Ok(0x1E),
        0x02DC => Ok(0x1F),
        0x2022 => Ok(0x80),
        0x2020 => Ok(0x81),
        0x2021 => Ok(0x82),
        0x2026 => Ok(0x83),
        0x2014 => Ok(0x84),
        0x2013 => Ok(0x85),
        0x0192 => Ok(0x86),
        0x2044 => Ok(0x87),
        0x2039 => Ok(0x88),
        0x203A => Ok(0x89),
        0x2212 => Ok(0x8A),
        0x2030 => Ok(0x8B),
        0x201E => Ok(0x8C),
        0x201C => Ok(0x8D),
        0x201D => Ok(0x8E),
        0x2018 => Ok(0x8F),

        0x2019 => Ok(0x90),
        0x201A => Ok(0x91),
        0x2122 => Ok(0x92),
        0xFB01 => Ok(0x93),
        0xFB02 => Ok(0x94),
        0x0141 => Ok(0x95),
        0x0152 => Ok(0x96),
        0x0160 => Ok(0x97),
        0x0178 => Ok(0x98),
        0x017D => Ok(0x99),
        0x0131 => Ok(0x9A),
        0x0142 => Ok(0x9B),
        0x0153 => Ok(0x9C),
        0x0161 => Ok(0x9D),
        0x017e => Ok(0x9E),

        0x20AC => Ok(0xA0),

        _ => Err(PDFDocEncodingError(chr)),
    }
}

/// Encode a string as PDFDocEncoding
pub fn pdf_doc_encode(input: &str) -> Result<Vec<u8>, PDFDocEncodingError> {
    input.chars().map(pdf_char_encode).collect()
}

/// Transliterate some non-representable characters
///
/// FIXME: Instead fall back to default mapping
fn pdf_char_encode_lossy(chr: char) -> Option<u8> {
    pdf_char_encode(chr).ok().or(match chr {
        'Α' => Some(b'A'),
        'Β' => Some(b'B'),
        'Γ' => Some(b'G'),
        'Δ' => Some(b'D'),
        'Ε' => Some(b'E'),
        'Ζ' => Some(b'Z'),
        'Η' => Some(b'H'),
        'Τ' => Some(b'T'),
        'Θ' => Some(b'I'),
        'Ι' => Some(b'I'),
        'Κ' => Some(b'K'),
        'Λ' => Some(b'L'),
        'Μ' => Some(b'M'),
        'Ν' => Some(b'N'),
        'Χ' | 'Ξ' => Some(b'X'),
        'Ο' => Some(b'O'),
        'Ψ' | 'Π' => Some(b'P'),
        'Ρ' => Some(b'R'),
        'Σ' => Some(b'S'),
        'Υ' => Some(b'Y'),
        'Φ' => Some(b'V'),
        'Ω' => Some(b'W'),

        'α' => Some(b'a'),
        'β' => Some(b'b'),
        'γ' => Some(b'g'),
        'δ' => Some(b'd'),
        'ε' => Some(b'e'),
        'ζ' => Some(b'z'),
        'η' => Some(b'h'),
        'τ' => Some(b't'),
        'θ' => Some(b'i'),
        'ι' => Some(b'i'),
        'κ' => Some(b'k'),
        'λ' => Some(b'l'),
        'μ' => Some(b'm'),
        'χ' | 'ξ' => Some(b'x'),
        'ο' => Some(b'o'),
        'ψ' | 'π' => Some(b'p'),
        'ρ' => Some(b'r'),
        'ς' | 'σ' => Some(b's'),
        'υ' => Some(b'y'),
        'φ' => Some(b'v'),
        'ω' => Some(b'w'),

        '‖' => Some(b'2'),
        _ => None,
    })
}

/// Encode a string as PDFDocEncoding, ignoring unconvertible characters
pub fn pdf_doc_encode_lossy(input: &str) -> Vec<u8> {
    input.chars().flat_map(pdf_char_encode_lossy).collect()
}
