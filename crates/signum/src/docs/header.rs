//! # (`0001`) The header of a document.

use std::fmt;

use nom::{
    bytes::streaming::take,
    combinator::{map, rest},
    error::ParseError,
    number::complete::be_u16,
    sequence::tuple,
    IResult,
};
use serde::{Deserialize, Serialize};

use crate::util::FourCC;

use super::Chunk;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// A GEMDOS date
#[serde(transparent)]
pub struct Date(pub u16);

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let year = (self.0 >> 9) + 1980;
        let month = (self.0 >> 5) & 0b1111;
        let day = self.0 & 0b11111;
        write!(f, "{:02}.{:02}.{:04}", day, month, year)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// A GEMDOS time
#[serde(transparent)]
pub struct Time(pub u16);

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hours = self.0 >> 11;
        let minutes = (self.0 >> 5) & 0b111111;
        let seconds = (self.0 & 0b11111) << 1;
        write!(f, "{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// A GEMDOS date and time
pub struct DateTime {
    /// The date part
    pub date: Date,
    /// The time part
    pub time: Time,
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.date, self.time)
    }
}

#[derive(Debug, Serialize)]
/// The header of a signum file
pub struct Header<'a> {
    /// Leading bytes, usually all zero
    #[serde(skip)]
    pub lead: &'a [u8],
    /// The created time
    pub ctime: DateTime,
    /// The last modified time
    pub mtime: DateTime,
    /// Trailing bytes, usually all zero
    #[serde(skip)]
    pub trail: &'a [u8],
}

/// Parse the time as a 16 bit integer
pub fn p_time<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Time, E> {
    map(be_u16, Time)(input)
}

/// Parse the time as a 16 bit integer
pub fn p_date<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Date, E> {
    map(be_u16, Date)(input)
}

/// Parse the time as a 16 bit integer
pub fn p_datetime<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], DateTime, E> {
    //map(be_u32, DateTime)(input)
    map(tuple((p_date, p_time)), |(date, time)| DateTime {
        date,
        time,
    })(input)
}

/// Parse the header (`0001`) chunk
pub fn parse_header<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Header, E> {
    let (rest, (lead, ctime, mtime, trail)) =
        tuple((take(0x48usize), p_datetime, p_datetime, rest))(input)?;
    Ok((
        rest,
        Header {
            lead,
            ctime,
            mtime,
            trail,
        },
    ))
}

impl<'a> Chunk<'a> for Header<'a> {
    const TAG: crate::util::FourCC = FourCC::_0001;

    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>,
    {
        parse_header(input)
    }
}
