//! # General utilities

use std::fmt::{Debug, Display};

pub mod bit_iter;
pub(crate) mod bit_writer;
pub mod data;

/// A `u16` that does not encode an integer
pub struct Bytes16(pub u16);

impl<'a> Debug for Bytes16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:04X}", self.0)
    }
}

impl<'a> Display for Bytes16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:04X}", self.0)
    }
}

/// A `u32` that does not encode an integer
pub struct Bytes32(pub u32);

impl<'a> Debug for Bytes32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:08X}", self.0)
    }
}

/// A simple byte buffer
#[derive(Hash)]
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
