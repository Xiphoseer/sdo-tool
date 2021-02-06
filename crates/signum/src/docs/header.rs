//! # (`0001`) The header of a document.

use std::fmt;

use nom::{
    bytes::streaming::take,
    combinator::{map, rest},
    number::complete::be_u16,
    sequence::tuple,
    IResult,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// A GEMDOS date
pub struct Date(pub u16);

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let year = (self.0 >> 9) + 1980;
        let month = (self.0 >> 5) & 0b1111;
        let day = self.0 & 0b11111;
        write!(f, "{:02}.{:02}.{:04}", day, month, year)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// A GEMDOS time
pub struct Time(pub u16);

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hours = self.0 >> 11;
        let minutes = (self.0 >> 5) & 0b111111;
        let seconds = (self.0 & 0b11111) << 1;
        write!(f, "{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug)]
/// The header of a signum file
pub struct Header<'a> {
    /// Leading bytes, usually all zero
    pub lead: &'a [u8],
    /// The created time
    pub ctime: DateTime,
    /// The last modified time
    pub mtime: DateTime,
    /// Trailing bytes, usually all zero
    pub trail: &'a [u8],
}

/// Parse the time as a 16 bit integer
pub fn p_time(input: &[u8]) -> IResult<&[u8], Time> {
    map(be_u16, Time)(input)
}

/// Parse the time as a 16 bit integer
pub fn p_date(input: &[u8]) -> IResult<&[u8], Date> {
    map(be_u16, Date)(input)
}

/// Parse the time as a 16 bit integer
pub fn p_datetime(input: &[u8]) -> IResult<&[u8], DateTime> {
    //map(be_u32, DateTime)(input)
    map(tuple((p_date, p_time)), |(date, time)| DateTime {
        date,
        time,
    })(input)
}

/// Parse the header (`0001`) chunk
pub fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
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
