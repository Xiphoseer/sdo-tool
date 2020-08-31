use nom::{multi::fill, number::complete::be_u8, IResult};
use std::fmt::{self, Debug, Display};

pub struct Key8([u8; 8]);

impl Display for Key8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in &self.0 {
            if *c == 0 {
                break;
            }
            write!(f, "{}", *c as char)?;
        }
        Ok(())
    }
}

impl Debug for Key8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Key8 as Display>::fmt(self, f)
    }
}

pub fn key8(input: &[u8]) -> IResult<&[u8], Key8> {
    let mut buf = [0u8; 8];
    let (input, _) = fill(be_u8, &mut buf)(input)?;
    Ok((input, Key8(buf)))
}

pub struct Bytes16(pub u16);

impl<'a> Debug for Bytes16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:04X}", self.0)
    }
}

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
