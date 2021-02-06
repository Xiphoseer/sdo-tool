//! # The Signum! document
//!
//! This module contains the datastructures and parsers for reading SDO files.

use nom::{
    combinator::map,
    number::complete::{be_u16, be_u32},
    IResult,
};

use crate::util::{Bytes16, Bytes32};
use fmt::Debug;
use std::{borrow::Cow, fmt};

pub mod container;
pub mod cset;
pub mod hcim;
pub mod header;
pub mod pbuf;
pub mod sysp;
pub mod tebu;

#[derive(Debug)]
struct SDoc<'a> {
    charsets: Vec<Cow<'a, str>>,
}

/// Take the next 16 bytes
pub fn bytes16(input: &[u8]) -> IResult<&[u8], Bytes16> {
    map(be_u16, Bytes16)(input)
}

/// Take the next 32 bytes
pub fn bytes32(input: &[u8]) -> IResult<&[u8], Bytes32> {
    map(be_u32, Bytes32)(input)
}
