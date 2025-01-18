//! # (`0001`) The header of a document.

use std::{borrow::Cow, fmt};

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

impl Date {
    /// Return the year, month and day parts
    pub fn to_ymd(&self) -> (u16, u16, u16) {
        let year = (self.0 >> 9) + 1980;
        let month = (self.0 >> 5) & 0b1111;
        let day = self.0 & 0b11111;
        (year, month, day)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (year, month, day) = self.to_ymd();
        write!(f, "{:02}.{:02}.{:04}", day, month, year)
    }
}

#[cfg(feature = "chrono")]
impl From<Date> for chrono::NaiveDate {
    fn from(value: Date) -> Self {
        let (year, month, day) = value.to_ymd();
        chrono::NaiveDate::from_ymd_opt(year.into(), month.into(), day.into()).unwrap()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// A GEMDOS time
#[serde(transparent)]
pub struct Time(pub u16);

impl Time {
    /// Return the hours, minutes and seconds in this (local) time
    pub fn to_hms(&self) -> (u16, u16, u16) {
        let hours = self.0 >> 11;
        let minutes = (self.0 >> 5) & 0b111111;
        let seconds = (self.0 & 0b11111) << 1;
        (hours, minutes, seconds)
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (hours, minutes, seconds) = self.to_hms();
        write!(f, "{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

#[cfg(feature = "chrono")]
impl From<Time> for chrono::NaiveTime {
    fn from(value: Time) -> Self {
        let (hours, minutes, seconds) = value.to_hms();
        chrono::NaiveTime::from_hms_opt(hours.into(), minutes.into(), seconds.into()).unwrap()
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

#[cfg(feature = "chrono")]
impl From<DateTime> for chrono::NaiveDateTime {
    fn from(value: DateTime) -> Self {
        chrono::NaiveDateTime::new(value.date.into(), value.time.into())
    }
}

#[derive(Debug, Serialize)]
/// The header of a signum file
pub struct Header<'a> {
    /// Leading bytes, usually all zero
    #[serde(skip)]
    pub lead: Cow<'a, [u8]>,
    /// The created time
    pub ctime: DateTime,
    /// The last modified time
    pub mtime: DateTime,
    /// Trailing bytes, usually all zero
    #[serde(skip)]
    pub trail: Cow<'a, [u8]>,
}

impl Header<'_> {
    /// Turn this instance into an owned variant
    pub fn into_owned(self) -> Header<'static> {
        Header {
            lead: Cow::Owned(self.lead.into_owned()),
            ctime: self.ctime,
            mtime: self.mtime,
            trail: Cow::Owned(self.trail.into_owned()),
        }
    }
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
pub fn parse_header<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Header<'a>, E> {
    let (rest, (lead, ctime, mtime, trail)) =
        tuple((take(0x48usize), p_datetime, p_datetime, rest))(input)?;
    Ok((
        rest,
        Header {
            lead: Cow::Borrowed(lead),
            ctime,
            mtime,
            trail: Cow::Borrowed(trail),
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
