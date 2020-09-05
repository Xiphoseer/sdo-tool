use crate::util::BIT_STRING;
use crate::Buf;
use nom::{
    bytes::complete::{tag, take},
    number::complete::{be_u32, be_u8},
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
pub struct ESet<'a> {
    pub buf1: Buf<'a>,
    pub chars: Vec<EChar<'a>>,
}

pub struct OwnedESet {
    inner: ESet<'static>,
    buffer: Vec<u8>,
}

impl Deref for OwnedESet {
    type Target = ESet<'static>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl OwnedESet {
    pub fn load(path: &Path) -> Result<Self, String> {
        let buffer = std::fs::read(path).unwrap();
        // SAFETY: this is safe, because `buffer` is plain data and
        // drop order between `inner` and `buffer` really doesn't matter.
        let input: &'static [u8] = unsafe { std::mem::transmute(&buffer[..]) };
        let (_, inner) = parse_eset(input).unwrap();
        Ok(Self { inner, buffer })
    }
}

#[derive(Debug)]
pub struct EChar<'a> {
    pub width: u8,
    pub height: u8,
    pub top: u8,
    d: u8,
    pub buf: &'a [u8],
}

impl<'a> ESet<'a> {
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
            println!("{}, {}x{}, {}", ch.top, wu, hu, ch.d);
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

pub fn parse_echar(input: &[u8]) -> IResult<&[u8], EChar> {
    let (input, top) = be_u8(input)?;
    let (input, height) = be_u8(input)?;
    let (input, width) = be_u8(input)?;
    let (input, d) = be_u8(input)?;
    let (input, buf) = take((height * 2) as usize)(input)?;
    Ok((
        input,
        EChar {
            width,
            height,
            top,
            d,
            buf,
        },
    ))
}

pub fn parse_eset(input: &[u8]) -> IResult<&[u8], ESet> {
    let (input, _) = tag(b"eset")(input)?;
    let (input, _) = tag(b"0001")(input)?;
    let (input, skip) = be_u32(input)?;

    let (input, buf1) = take(skip as usize)(input)?;

    let (input, len) = be_u32(input)?;

    let (input, mut offset_buf) = take((skip - 1) as usize * 4)(input)?;
    let (input, char_buf) = take(len as usize)(input)?;

    let mut chars = Vec::with_capacity(skip as usize);
    chars.push(EChar {
        width: 0,
        height: 0,
        top: 0,
        d: 0,
        buf: &[],
    });

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
