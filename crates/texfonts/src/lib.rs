use std::{borrow::Cow, fmt, marker::PhantomData};

use fmt::Debug;
use nom::{
    bytes::complete::take,
    combinator::map,
    error::ParseError,
    number::complete::{be_i16, be_i32, be_i8, be_u16, be_u32, be_u8, le_u8},
    sequence::tuple,
    IResult, Parser,
};

#[derive(Debug)]
pub struct PackedFont<'i> {
    events: Vec<Event<'i>>,
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

pub fn be_u24<'i, E>(input: &'i [u8]) -> IResult<&'i [u8], u32, E>
where
    E: ParseError<&'i [u8]>,
{
    map(tuple((be_u8, be_u16)), |(high, low)| {
        (u32::from(high) << 16) + u32::from(low)
    })(input)
}

#[derive(Debug)]
pub enum Flag {
    Short,
    Extended,
    Long,
}

impl<'i, E> Parser<&'i [u8], CharPreamble, E> for Flag
where
    E: ParseError<&'i [u8]> + 'i,
{
    fn parse(&mut self, input: &'i [u8]) -> IResult<&'i [u8], CharPreamble, E> {
        match self {
            Self::Short => {
                let (input, (tfm, dm, w, h, hoff, voff)) =
                    tuple((be_u24, be_u8, be_u8, be_u8, be_i8, be_i8))(input)?;
                Ok((
                    input,
                    CharPreamble {
                        tfm,
                        dx: u32::from(dm) << 16,
                        dy: 0,
                        w: u32::from(w),
                        h: u32::from(h),
                        hoff: hoff.into(),
                        voff: voff.into(),
                    },
                ))
            }
            Self::Extended => {
                let (input, (tfm, dm, w, h, hoff, voff)) =
                    tuple((be_u24, be_u16, be_u16, be_u16, be_i16, be_i16))(input)?;
                Ok((
                    input,
                    CharPreamble {
                        tfm,
                        dx: u32::from(dm) << 16,
                        dy: 0,
                        w: u32::from(w),
                        h: u32::from(h),
                        hoff: hoff.into(),
                        voff: voff.into(),
                    },
                ))
            }
            Self::Long => {
                let (input, (tfm, dx, dy, w, h, hoff, voff)) =
                    tuple((be_u32, be_u32, be_u32, be_u32, be_u32, be_i32, be_i32))(input)?;
                Ok((
                    input,
                    CharPreamble {
                        tfm,
                        dx,
                        dy,
                        w,
                        h,
                        hoff,
                        voff,
                    },
                ))
            }
        }
    }
}

#[derive(Debug)]
pub struct CharPreamble {
    tfm: u32,
    dx: u32,
    dy: u32,
    w: u32,
    h: u32,
    hoff: i32,
    voff: i32,
}

#[derive(Debug)]
pub enum Command<'i> {
    Pre(Preamble<'i>),
    Post,
    NoOp,
    Unassigned(u8),
}

#[derive(Debug)]
pub struct Character<'i> {
    pub dyn_f: u8,
    pub first_run_black: bool,
    pub cc: u32,
    pub fl: Flag,
    pub bytes: &'i [u8],
}

#[derive(Debug)]
pub enum Event<'i> {
    Command(Command<'i>),
    Character(Character<'i>),
}

pub fn p_next<'i, E>(input: &'i [u8]) -> IResult<&'i [u8], Event<'i>, E>
where
    E: ParseError<&'i [u8]>,
{
    let (input, flag) = be_u8(input)?;
    let (input, event) = match flag {
        0..=239 => {
            let dyn_f = flag >> 4;
            let first_run_black = flag & 0b1000 != 0;

            let (input, pl, cc, fl) = match flag & 0b111 {
                0..=3 => {
                    let (input, (pl, cc)) = tuple((be_u8, be_u8))(input)?;
                    let pl = (u32::from(flag & 0b11) << 8) + u32::from(pl);
                    (input, pl, u32::from(cc), Flag::Short)
                }
                4..=6 => {
                    let (input, (pl, cc)) = tuple((be_u16, be_u8))(input)?;
                    let pl = (u32::from(flag & 0b11) << 16) + u32::from(pl);
                    (input, pl, u32::from(cc), Flag::Extended)
                }
                _ => {
                    let (input, (pl, cc)) = tuple((be_u32, be_u32))(input)?;
                    (input, pl, cc, Flag::Long)
                }
            };
            let (input, bytes) = take(pl as usize)(input)?;
            (
                input,
                Event::Character(Character {
                    dyn_f,
                    first_run_black,
                    cc,
                    fl,
                    bytes,
                }),
            )
        }
        240 => todo!("pk_xxx1"),
        241 => todo!("pk_xxx2"),
        242 => todo!("pk_xxx3"),
        243 => todo!("pk_xxx4"),
        244 => todo!("pk_yyy"),
        245 => (input, Event::Command(Command::Post)),
        246 => (input, Event::Command(Command::NoOp)),
        247 => p_preamble(input).map(|(i, p)| (i, Event::Command(Command::Pre(p))))?,
        _ => (input, Event::Command(Command::Unassigned(flag))),
    };
    Ok((input, event))
}

pub struct Decoder<'i, E: ParseError<&'i [u8]>> {
    inner: &'i [u8],
    _e: PhantomData<fn() -> E>,
}

impl<'i, E: ParseError<&'i [u8]>> Decoder<'i, E> {
    pub fn new(inner: &'i [u8]) -> Self {
        Self {
            inner,
            _e: PhantomData,
        }
    }
}

impl<'i, E> Iterator for Decoder<'i, E>
where
    E: ParseError<&'i [u8]>,
{
    type Item = Result<Event<'i>, nom::Err<E>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.is_empty() {
            return None;
        }
        match p_next(self.inner) {
            Ok((input, event)) => {
                self.inner = input;
                Some(Ok(event))
            }
            Err(err) => {
                // make sure to skip the rest
                self.inner = self.inner.split_at(self.inner.len()).1;
                Some(Err(err))
            }
        }
    }
}

pub fn p_pk<'i, E>(input: &'i [u8]) -> IResult<&'i [u8], PackedFont<'i>, E>
where
    E: ParseError<&'i [u8]>,
{
    let (input, first) = p_next(input)?;
    let (input, second) = p_next(input)?;

    Ok((
        input,
        PackedFont {
            events: vec![first, second],
        },
    ))
}
