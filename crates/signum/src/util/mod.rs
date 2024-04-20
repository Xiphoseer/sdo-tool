//! # General utilities

use std::{
    fmt::{self, Debug, Display},
    ops::Deref,
};

use bstr::{BStr, ByteSlice};
use serde::{Deserialize, Serialize};

pub mod bit_iter;
pub(crate) mod bit_writer;
pub mod data;

/// A `u16` that does not encode an integer
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Bytes16(pub u16);

impl Debug for Bytes16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:04X}", self.0)
    }
}

impl Display for Bytes16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:04X}", self.0)
    }
}

/// A `u32` that does not encode an integer
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Bytes32(pub u32);

impl Debug for Bytes32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:08X}", self.0)
    }
}

/// A simple byte buffer
#[derive(Hash, Serialize)]
#[serde(transparent)]
pub struct Buf<'a>(pub &'a [u8]);

impl<'a> Debug for Buf<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max = self.0.len();
        if f.alternate() {
            writeln!(f, "Buf[{}]", max)?;
            write!(f, "  ")?;
        }
        for (index, byte) in self.0.iter().cloned().enumerate() {
            write!(f, "{:02X}", byte)?;
            if index + 1 < max {
                if f.alternate() && (index + 1) % 16 == 0 && index > 0 {
                    write!(f, "\n  ")?;
                } else {
                    write!(f, " ")?;
                }
            }
        }
        Ok(())
    }
}

impl<'a> Display for Buf<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Buf as Debug>::fmt(self, f)
    }
}

/// A four character code
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FourCC(pub(crate) [u8; 4]);

impl FourCC {
    /// `sdoc` - Signum Document
    pub const SDOC: FourCC = FourCC(*b"sdoc");
    /// `bimc` - Hardcopy Image File (standalone)
    pub const BIMC: FourCC = FourCC(*b"bimc");
    /// `eset` - Editor Font
    pub const ESET: FourCC = FourCC(*b"eset");
    /// `ps24` - 24-Needle Printer Font
    pub const PS24: FourCC = FourCC(*b"ps24");
    /// `ps09` - 24-Needle Printer Font
    pub const PS09: FourCC = FourCC(*b"ps09");
    /// `ls30` - 24-Needle Printer Font
    pub const LS30: FourCC = FourCC(*b"ls30");

    /// `0001`
    pub const _0001: FourCC = FourCC(*b"0001");
    /// `cset`
    pub const _CSET: FourCC = FourCC(*b"cset");
    /// `sysp`
    pub const _SYSP: FourCC = FourCC(*b"sysp");
    /// `pbuf`
    pub const _PBUF: FourCC = FourCC(*b"pbuf");
    /// `tebu`
    pub const _TEBU: FourCC = FourCC(*b"tebu");
    /// `hcim`
    pub const _HCIM: FourCC = FourCC(*b"hcim");
    /// `pl01`
    pub const _PL01: FourCC = FourCC(*b"pl01");
    /// `syp2`
    pub const _SYP2: FourCC = FourCC(*b"syp2");

    /// Return this FourCC as a slice of bytes
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Return this FourCC as a [`bstr::BStr`]
    pub fn as_bstr(&self) -> &BStr {
        self.0.as_bstr()
    }
}

impl fmt::Debug for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.as_bstr(), f)
    }
}

impl fmt::Display for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.as_bstr(), f)
    }
}

impl Deref for FourCC {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A 16 bit position
pub struct Pos {
    /// horizontal
    pub x: u16,
    /// vertical
    pub y: u16,
}

impl Pos {
    /// Create a new point
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}
