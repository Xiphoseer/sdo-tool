use std::{
    io::{self, Write},
    path::PathBuf,
};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opts {
    file: PathBuf,
}

#[rustfmt::skip]
const ZERO: [u8; 8] = [
    0b00111100,
    0b01000010,
    0b01000010,
    0b01000010,
    0b01000010,
    0b01000010,
    0b01000010,
    0b00111100,
];

#[rustfmt::skip]
const _ONE: [u8; 8] = [
    0b00011000,
    0b00101000,
    0b01001000,
    0b00001000,
    0b00001000,
    0b00001000,
    0b00001000,
    0b01111110,
];

#[rustfmt::skip]
const ONE: [u8; 8] = [
    0b00001000,
    0b00011000,
    0b00101000,
    0b00001000,
    0b00001000,
    0b00001000,
    0b00001000,
    0b00001000,
];

#[rustfmt::skip]
const TWO: [u8; 8] = [
    0b01111100,
    0b00000010,
    0b00000010,
    0b00000010,
    0b00111100,
    0b01000000,
    0b01000000,
    0b01111110,
];

#[rustfmt::skip]
const THREE: [u8; 8] = [
    0b01111100,
    0b00000010,
    0b00000010,
    0b00111100,
    0b00000010,
    0b00000010,
    0b00000010,
    0b01111100,
];

#[rustfmt::skip]
const FOUR: [u8; 8] = [
    0b00000000,
    0b01000010,
    0b01000010,
    0b01000010,
    0b00111110,
    0b00000010,
    0b00000010,
    0b00000010,
];

#[rustfmt::skip]
const FIVE: [u8; 8] = [
    0b01111110,
    0b01000000,
    0b01000000,
    0b01000000,
    0b00111100,
    0b00000010,
    0b00000010,
    0b01111100,
];

#[rustfmt::skip]
const SIX: [u8; 8] = [
    0b00011110,
    0b00100000,
    0b01000000,
    0b01000000,
    0b00111100,
    0b01000010,
    0b01000010,
    0b00111100,
];

#[rustfmt::skip]
const SEVEN: [u8; 8] = [
    0b01111110,
    0b00000010,
    0b00000100,
    0b00001000,
    0b00001000,
    0b00010000,
    0b00010000,
    0b00010000,
];

#[rustfmt::skip]
const EIGHT: [u8; 8] = [
    0b00111100,
    0b01000010,
    0b01000010,
    0b01000010,
    0b00111100,
    0b01000010,
    0b01000010,
    0b00111100,
];

#[rustfmt::skip]
const NINE: [u8; 8] = [
    0b00111100,
    0b01000010,
    0b01000010,
    0b00111110,
    0b00000010,
    0b00000010,
    0b01000010,
    0b00111100,
];

#[rustfmt::skip]
const TEN: [u8; 8] = [
    0b00111100,
    0b01000010,
    0b01000010,
    0b01000010,
    0b01111110,
    0b01000010,
    0b01000010,
    0b01000010,
];

#[rustfmt::skip]
const ELEVEN: [u8; 8] = [
    0b01111100,
    0b01000010,
    0b01000010,
    0b01000010,
    0b01111100,
    0b01000010,
    0b01000010,
    0b01111100,
];

#[rustfmt::skip]
const TWELVE: [u8; 8] = [
    0b00011110,
    0b00100000,
    0b01000000,
    0b01000000,
    0b01000000,
    0b01000000,
    0b00100000,
    0b00011110,
];

#[rustfmt::skip]
const THIRTEEN: [u8; 8] = [
    0b01111000,
    0b01000100,
    0b01000010,
    0b01000010,
    0b01000010,
    0b01000010,
    0b01000100,
    0b01111000,
];

#[rustfmt::skip]
const FOURTEEN: [u8; 8] = [
    0b01111110,
    0b01000000,
    0b01000000,
    0b01111110,
    0b01000000,
    0b01000000,
    0b01000000,
    0b01111110,
];

#[rustfmt::skip]
const FIFTEEN: [u8; 8] = [
    0b01111110,
    0b01000000,
    0b01000000,
    0b01111110,
    0b01000000,
    0b01000000,
    0b01000000,
    0b01000000,
];

const NUMS: [[u8; 8]; 16] = [
    ZERO, ONE, TWO, THREE, FOUR, FIVE, SIX, SEVEN, EIGHT, NINE, TEN, ELEVEN, TWELVE, THIRTEEN,
    FOURTEEN, FIFTEEN,
];

fn write_font(buf: &mut Vec<u8>) -> io::Result<()> {
    buf.write_all(b"eset0001")?;
    buf.write_all(&128u32.to_be_bytes())?;
    for _ in 0..32 {
        buf.write_all(&[0, 0, 0, 0])?;
    }
    let h: u8 = 19;
    let clen = (h * 2) as u32 + 4;
    let max: u32 = clen * 127 + 4;
    buf.write_all(&max.to_be_bytes())?;
    let mut off: u32 = 4;
    for _ in 1..128 {
        buf.write_all(&off.to_be_bytes())?;
        off += clen;
    }
    buf.write_all(&[0, 0, 0, 0])?; // 0 byte
    for i in 1..128 {
        let high = NUMS[(i / 16) as usize];
        let low = NUMS[(i % 16) as usize];
        buf.write_all(&[0, h, 16, 0])?;
        for &x in &high {
            buf.write_all(&[x, 0])?;
        }
        buf.write_all(&[0, 0, 255, 0, 0, 0])?;
        for &x in &low {
            buf.write_all(&[x, 0])?;
        }
    }
    Ok(())
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let opts = Opts::from_args();

    let mut contents: Vec<u8> = Vec::new();
    write_font(&mut contents)?;
    std::fs::write(opts.file, contents)?;

    Ok(())
}
