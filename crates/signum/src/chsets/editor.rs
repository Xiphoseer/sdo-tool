//! # The editor fonts

use super::{error::ChsetSizeError, FontKind, LoadError};
use crate::util::{data::BIT_STRING, Buf, FileFormatKind};
use log::warn;
use nom::{
    bytes::complete::{tag, take},
    number::complete::{be_u32, u8},
    IResult,
};
use std::{borrow::Cow, io, ops::Deref, path::Path};

const BORDER: [&str; 22] = [
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
    // oversized advance
    "+----------------+|",
    "+----------------+-|",
    "+----------------+--|",
    "+----------------+---|",
    "+----------------+----|",
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
        Self::load_from_buf(buffer)
    }

    /// Load an editor charset
    pub fn load_from_buf(buffer: Vec<u8>) -> Result<Self, LoadError> {
        // SAFETY: this is safe, because `buffer` is plain data and
        // drop order between `inner` and `buffer` really doesn't matter.
        let input: &'static [u8] = unsafe { std::mem::transmute(&buffer[..]) };
        let (_, inner) = parse_eset(input).unwrap();
        Ok(Self { inner, buffer })
    }
}

#[derive(Debug, PartialEq)]
/// A single editor charset character
pub struct EChar<'a> {
    /// The width of the glyph (in document coordinates)
    pub width: u8,
    /// The height of the glyph
    pub height: u8,
    /// The distance of the top edge of the stored glyph from the top of the available box
    pub top: u8,
    /// The buffer that contains the pixels
    pub buf: Cow<'a, [u8]>,
}

impl<'a> EChar<'a> {
    /// Create a new glyph from a bitmap and metrics
    pub const fn new(
        width: u8,
        height: u8,
        top: u8,
        buf: &'a [u8],
    ) -> Result<Self, ChsetSizeError> {
        let expected = height as usize * 2;
        if buf.len() == expected {
            Ok(Self {
                width,
                height,
                top,
                buf: Cow::Borrowed(buf),
            })
        } else {
            Err(ChsetSizeError::UnexpectedBitmapSize {
                expected,
                actual: buf.len(),
            })
        }
    }
}

impl EChar<'static> {
    /// Create a new, owned [EChar]
    pub fn new_owned(width: u8, height: u8, top: u8, buf: Vec<u8>) -> Result<Self, ChsetSizeError> {
        let expected = height as usize * 2;
        if buf.len() == expected {
            Ok(Self {
                width,
                height,
                top,
                buf: Cow::Owned(buf),
            })
        } else {
            Err(ChsetSizeError::UnexpectedBitmapSize {
                expected,
                actual: buf.len(),
            })
        }
    }
}

impl EChar<'_> {
    /// Print the character to the console
    pub fn print(&self) {
        let wu = self.width as usize;
        let hu = self.height as usize;
        println!("{}, {}x{}", self.top, wu, hu);
        let border = BORDER[wu];
        println!("{}", border);
        for _ in 1..self.top {
            println!("|                |");
        }
        if self.top > 0 {
            println!("-                -");
        }
        for i in 0..hu {
            let left = self.buf[2 * i] as usize;
            let right = self.buf[2 * i + 1] as usize;
            println!("|{}{}|", &BIT_STRING[left], &BIT_STRING[right]);
        }
        let rest = 24 - self.top - self.height;
        if rest > 0 {
            println!("-                -");
        }
        for _ in 1..rest {
            println!("|                |");
        }
        println!("{}", border);
    }
}

/// The special NULL char
pub const ECHAR_NULL: EChar<'static> = EChar {
    width: 0,
    height: 0,
    top: 0,
    buf: Cow::Borrowed(&[]),
};

impl ESet<'_> {
    /// Print a representation of the charset to the console
    ///
    /// FIXME: make this generic over `io::Write` or `fmt::Write`?
    pub fn print(&self) {
        let capacity = self.chars.len();
        let mut widths = Vec::with_capacity(capacity);
        let mut skips = Vec::with_capacity(capacity);
        for (index, ch) in self.chars.iter().enumerate() {
            println!("\nchar[{}]", index);
            widths.push(ch.width);
            skips.push(ch.top);
            ch.print();
        }
        println!();
        println!("pub const WIDTH: [u8; 128] = [");
        for (i, w) in widths.iter().cloned().enumerate() {
            print!("{:3},", w);
            if i % 16 == 15 {
                println!();
            } else {
                print!(" ");
            }
        }
        println!("];");
        println!("pub const SKIP: [u8; 128] = [");
        for (i, s) in skips.iter().cloned().enumerate() {
            print!("{:3},", s);
            if i % 16 == 15 {
                println!();
            } else {
                print!(" ");
            }
        }
        println!("];");
    }

    /// Write editor font to a file
    pub fn write_to<W: io::Write>(&self, buf: &mut W) -> io::Result<()> {
        buf.write_all(FontKind::Editor.magic().as_slice())?;
        buf.write_all(b"0001")?;
        buf.write_all(&128u32.to_be_bytes())?;
        for _ in 0..32 {
            buf.write_all(&[0, 0, 0, 0])?;
        }
        let mut off: u32 = 4;
        let mut offsets = Vec::with_capacity(128);
        for i in 1..128 {
            offsets.push(off);
            let c = &self.chars[i];
            off += (c.height as u32 * 2) + 4;
        }
        let max: u32 = off;
        buf.write_all(&max.to_be_bytes())?;
        for off in offsets {
            buf.write_all(&off.to_be_bytes())?;
        }
        for i in 0..128 {
            let c = &self.chars[i];
            buf.write_all(&[c.top, c.height, c.width, 0])?;
            assert_eq!(c.buf.len(), c.height as usize * 2);
            buf.write_all(c.buf.as_ref())?;
        }
        Ok(())
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
            buf: Cow::Borrowed(buf),
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

    for _i in 1..skip {
        let (rest, offset) = be_u32(offset_buf)?;
        if (offset as usize) + 4 > char_buf.len() {
            warn!(
                "eset: Offset {offset} out of bounds (len {}, at {_i})",
                char_buf.len()
            );
            chars.push(ECHAR_NULL);
        } else {
            let at_offset = &char_buf[offset as usize..];
            let (_, echar) = parse_echar(at_offset)?;
            chars.push(echar);
        }
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
