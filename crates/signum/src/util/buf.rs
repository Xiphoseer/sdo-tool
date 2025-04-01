use std::fmt;

use serde::Serialize;

/// A simple byte buffer
#[derive(Hash, Serialize, Clone, Copy)]
#[serde(transparent)]
pub struct Buf<'a>(pub &'a [u8]);

impl fmt::Debug for Buf<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl fmt::Display for Buf<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Buf as fmt::Debug>::fmt(self, f)
    }
}
