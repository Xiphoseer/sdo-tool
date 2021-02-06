//! # The editor fonts

use super::LoadError;
use crate::util::{data::BIT_STRING, Buf};
use nom::{
    bytes::complete::{tag, take},
    number::complete::{be_u32, u8},
    IResult,
};
use std::{ops::Deref, path::Path};

const BORDER: [&str; 17] = [
    "+|---------------+",
    "+-|--------------+",
    "+--|-------------+",
    "+---|------------+",
    "+----|-----------+",
    "+-----|----------+",
    "+------|---------+",
    "+-------|--------+",
    "+--------|-------+",
    "+---------|------+",
    "+----------|-----+",
    "+-----------|----+",
    "+------------|---+",
    "+-------------|--+",
    "+--------------|-+",
    "+---------------|+",
    "+----------------+",
];

#[derive(Debug)]
/// A single editor font
pub struct ESet<'a> {
    /// The leading buffer / header
    pub buf1: Buf<'a>,
    /// The chars in this charset
    pub chars: Vec<EChar<'a>>,
}

/// An owned version of the editor font
pub struct OwnedESet {
    inner: ESet<'static>,
    #[allow(unused)]
    buffer: Vec<u8>,
}

impl Deref for OwnedESet {
    type Target = ESet<'static>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl OwnedESet {
    /// Load an editor charset
    pub fn load(path: &Path) -> Result<Self, LoadError> {
        let buffer = std::fs::read(path)?;
        // SAFETY: this is safe, because `buffer` is plain data and
        // drop order between `inner` and `buffer` really doesn't matter.
        let input: &'static [u8] = unsafe { std::mem::transmute(&buffer[..]) };
        let (_, inner) = parse_eset(input).unwrap();
        Ok(Self { inner, buffer })
    }
}

#[derive(Debug)]
/// A single editor charset character
pub struct EChar<'a> {
    /// The width of the glyph (in document coordinates)
    pub width: u8,
    /// The height of the glyph
    pub height: u8,
    /// The distance of the top edge of the stored glyph from the top of the available box
    pub top: u8,
    /// The buffer that contains the pixels
    pub buf: &'a [u8],
}

/// The special NULL char
pub const ECHAR_NULL: EChar<'static> = EChar {
    width: 0,
    height: 0,
    top: 0,
    buf: &[],
};

impl<'a> ESet<'a> {
    /// Print a representation of the charset to the console
    ///
    /// FIXME: make this generic over `io::Write` or `fmt::Write`?
    pub fn print(&self) {
        let capacity = self.chars.len();
        let mut widths = Vec::with_capacity(capacity);
        let mut skips = Vec::with_capacity(capacity);
        for (index, ch) in self.chars.iter().enumerate() {
            println!("\nchar[{}]", index);
            let wu = ch.width as usize;
            let hu = ch.height as usize;
            widths.push(ch.width);
            skips.push(ch.top);
            println!("{}, {}x{}", ch.top, wu, hu);
            let border = BORDER[wu];
            println!("{}", border);
            for _ in 1..ch.top {
                println!("|                |");
            }
            if ch.top > 0 {
                println!("-                -");
            }
            for i in 0..hu {
                let left = ch.buf[2 * i] as usize;
                let right = ch.buf[2 * i + 1] as usize;
                println!("|{}{}|", &BIT_STRING[left], &BIT_STRING[right]);
            }
            let rest = 24 - ch.top - ch.height;
            if rest > 0 {
                println!("-                -");
            }
            for _ in 1..rest {
                println!("|                |");
            }
            println!("{}", border);
        }
        println!();
        println!("pub const WIDTH: [u8; 128] = [");
        print!("  0, ");
        for (i, w) in widths.iter().cloned().enumerate() {
            if i % 16 == 15 {
                println!();
            }
            print!("{:3},", w);
            if i % 16 != 14 {
                print!(" ");
            }
        }
        println!();
        println!("];");
        println!("pub const SKIP: [u8; 128] = [");
        print!("  0, ");
        for (i, s) in skips.iter().cloned().enumerate() {
            if i % 16 == 15 {
                println!();
            }
            print!("{:3},", s);
            if i % 16 != 14 {
                print!(" ");
            }
        }
        println!();
        println!("];");
    }
}

/// Parse a single editor char
pub fn parse_echar(input: &[u8]) -> IResult<&[u8], EChar> {
    let (input, top) = u8(input)?;
    let (input, height) = u8(input)?;
    let (input, width) = u8(input)?;
    let (input, _d) = u8(input)?;
    let (input, buf) = take((height * 2) as usize)(input)?;
    Ok((
        input,
        EChar {
            width,
            height,
            top,
            buf,
        },
    ))
}

/// Parse a full editor charset file.
///
/// This method checks for the magic bytes `eset0001`
pub fn parse_eset(input: &[u8]) -> IResult<&[u8], ESet> {
    let (input, _) = tag(b"eset")(input)?;
    let (input, _) = tag(b"0001")(input)?;
    let (input, skip) = be_u32(input)?;

    let (input, buf1) = take(skip as usize)(input)?;

    let (input, len) = be_u32(input)?;

    let (input, mut offset_buf) = take((skip - 1) as usize * 4)(input)?;
    let (input, char_buf) = take(len as usize)(input)?;

    let mut chars = Vec::with_capacity(skip as usize);
    chars.push(ECHAR_NULL);

    for _ in 1..skip {
        let (rest, offset) = be_u32(offset_buf)?;
        let (_, echar) = parse_echar(&char_buf[offset as usize..])?;
        chars.push(echar);
        offset_buf = rest;
    }

    Ok((
        input,
        ESet {
            buf1: Buf(buf1),
            chars,
        },
    ))
}
