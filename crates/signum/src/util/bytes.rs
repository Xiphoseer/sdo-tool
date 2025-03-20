use std::fmt;

use serde::{Deserialize, Serialize};

/// A `u16` that does not encode an integer
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Bytes16(pub u16);

impl Bytes16 {
    /// Return the bytes in big endian order
    pub fn to_bytes(&self) -> [u8; 2] {
        self.0.to_be_bytes()
    }
}

impl fmt::Debug for Bytes16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:04X}", self.0)
    }
}

impl fmt::Display for Bytes16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:04X}", self.0)
    }
}

/// A `u32` that does not encode an integer
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Bytes32(pub u32);

impl fmt::Debug for Bytes32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:08X}", self.0)
    }
}
