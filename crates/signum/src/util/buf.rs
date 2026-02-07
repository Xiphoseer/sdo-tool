use std::fmt;

use serde::Serialize;

/// A simple byte buffer
#[derive(Hash, Serialize, Clone)]
#[serde(transparent)]
pub struct Cow<'a>(std::borrow::Cow<'a, [u8]>);

impl fmt::Debug for Cow<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Buf as fmt::Debug>::fmt(&Buf(self.0.as_ref()), f)
    }
}

impl<'a> Cow<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self(std::borrow::Cow::Borrowed(bytes))
    }

    pub(crate) fn new_owned(bytes: Vec<u8>) -> Self {
        Self(std::borrow::Cow::Owned(bytes))
    }
}

impl fmt::Display for Cow<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Cow as fmt::Debug>::fmt(self, f)
    }
}

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
        for (ri, row) in self.0.chunks(16).enumerate() {
            if ri > 0 && f.alternate() {
                write!(f, "\n  ")?;
            }
            for (bi, byte) in row.iter().enumerate() {
                if bi > 0 {
                    write!(f, " ")?;
                }
                write!(f, "{:02X}", byte)?;
            }

            if f.alternate() {
                write!(f, "  ")?;
                for byte in row.iter() {
                    if byte.is_ascii_alphanumeric() {
                        write!(f, "{}", *byte as char)?;
                    } else {
                        write!(f, ".")?;
                    }
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
