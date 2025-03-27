use std::fmt;

use nom::{
    bytes::complete::take,
    combinator::{map, rest},
    error::ParseError,
    number::complete::be_u16,
    sequence::{pair, tuple},
    IResult,
};

use crate::util::{Buf, V3Chunk};

use super::TAG_SDOC3;

/// Header of a document
#[derive(Debug)]
#[allow(dead_code)]
pub struct Header<'a> {
    lead: Buf<'a>,
    /// Create time
    pub ctime: DateTime,
    /// Modified time
    pub mtime: DateTime,
    tail: Buf<'a>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Full date
pub struct Date {
    year: u16,
    month: u16,
    day: u16,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Time of day
pub struct Time {
    hour: u16,
    minute: u16,
    second: u16,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// [Date] and [Time]
pub struct DateTime {
    date: Date,
    time: Time,
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.date.fmt(f)?;
        f.write_str("T")?;
        self.time.fmt(f)?;
        Ok(())
    }
}

fn parse_date<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Date, E> {
    let (input, (year, month, day)) = tuple((be_u16, be_u16, be_u16))(input)?;
    Ok((input, Date { year, month, day }))
}

fn parse_time<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Time, E> {
    let (input, (hour, minute, second)) = tuple((be_u16, be_u16, be_u16))(input)?;
    Ok((
        input,
        Time {
            hour,
            minute,
            second,
        },
    ))
}

fn parse_datetime<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], DateTime, E> {
    let (input, (date, time)) = pair(parse_date, parse_time)(input)?;
    Ok((input, DateTime { date, time }))
}

impl<'a> V3Chunk<'a> for Header<'a> {
    const CONTEXT: &'static str = "sdoc 03";
    const TAG: &'static [u8; 12] = TAG_SDOC3;

    fn parse<E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Header<'a>, E> {
        let (input, lead) = map(take(40usize), Buf)(input)?;
        let (input, ctime) = parse_datetime(input)?;
        let (input, mtime) = parse_datetime(input)?;
        let (input, tail) = map(rest, Buf)(input)?;
        Ok((
            input,
            Header {
                lead,
                ctime,
                mtime,
                tail,
            },
        ))
    }
}
