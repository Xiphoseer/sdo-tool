use std::{borrow::Cow, fmt};

use fmt::Debug;
use nom::{
    bytes::complete::{tag, take},
    combinator::map,
    error::ParseError,
    number::complete::{be_u16, be_u32, be_u8, le_u8},
    sequence::tuple,
    IResult,
};

#[derive(Debug)]
pub struct PackedFont<'i> {
    preamble: Preamble<'i>,
}

#[derive(Debug)]
pub struct Preamble<'i> {
    version: u8,
    x: Cow<'i, str>,
    ds: DesignSize,
    cs: u32,
    hppp: u32,
    vppp: u32,
}

pub struct DesignSize(u32);

impl fmt::Debug for DesignSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0 >> 16)?;
        let rem = self.0 & 0xffff;
        if rem > 0 {
            write!(f, ".todo")?;
        }
        Ok(())
    }
}

pub fn p_preamble<'i, E>(input: &'i [u8]) -> IResult<&'i [u8], Preamble<'i>, E>
where
    E: ParseError<&'i [u8]>,
{
    let (input, version) = le_u8(input)?;
    let (input, k) = le_u8(input)?;
    let (input, x) = take(k as usize)(input)?;
    let (input, ds) = map(be_u32, DesignSize)(input)?;
    let (input, cs) = be_u32(input)?;
    let (input, hppp) = be_u32(input)?;
    let (input, vppp) = be_u32(input)?;
    Ok((
        input,
        Preamble {
            version,
            x: String::from_utf8_lossy(x),
            ds,
            cs,
            hppp,
            vppp,
        },
    ))
}

#[allow(non_camel_case_types)]
pub struct u24(u8, u16);

pub fn be_u24<'i, E>(input: &'i [u8]) -> IResult<&'i [u8], u24, E>
where
    E: ParseError<&'i [u8]>,
{
    map(tuple((be_u8, be_u16)), |(high, low)| u24(high, low))(input)
}

impl fmt::Debug for u24 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (u32::from(self.1) + (u32::from(self.0) << 16)).fmt(f)
    }
}

#[derive(Debug)]
pub struct CharPreamble {
    pl: u8,
    cc: u8,
    tfm: u24, // u24
    dm: u8,
    w: u8,
    h: u8,
}

pub fn p_char_preamble_short<'i, E>(input: &'i [u8]) -> IResult<&'i [u8], CharPreamble, E>
where
    E: ParseError<&'i [u8]>,
{
    map(
        tuple((be_u8, be_u8, be_u24, be_u8, be_u8, be_u8)),
        |(pl, cc, tfm, dm, w, h)| CharPreamble {
            pl,
            cc,
            tfm,
            dm,
            w,
            h,
        },
    )(input)
}

pub fn p_pk<'i, E>(input: &'i [u8]) -> IResult<&'i [u8], PackedFont<'i>, E>
where
    E: ParseError<&'i [u8]>,
{
    let (input, _) = tag(&[247u8])(input)?;
    let (input, preamble) = p_preamble(input)?;
    let (input, next) = be_u8(input)?;
    let dyn_f = next >> 4;
    let input = if dyn_f < 15 {
        let first_run_black = next & 0b1000 != 0;
        println!("dyn_f: {}", dyn_f);
        println!("first_run_black: {}", first_run_black);
        let (input, cp) = match next & 0b111 {
            0..=3 => {
                println!("short form");
                p_char_preamble_short(input)?
            }
            4..=6 => {
                println!("extended form");
                todo!()
            }
            _ => {
                println!("long form");
                todo!()
            }
        };
        println!("{:?}", cp);
        input
    } else {
        input
    };
    Ok((input, PackedFont { preamble }))
}
