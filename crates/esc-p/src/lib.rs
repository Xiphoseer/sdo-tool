#![warn(missing_docs)]
//! # ESC/P in Rust
//!
//! This crate implements a (currently very small) subset of ESC/P to support
//! analyzing how applications print and to implement virtual printers.
//!
//! This crate was created as part of the [SDO-Toolbox](https://sdo.dseiler.eu)
//! project.

use std::{fmt, io};

/// # ESC/P decoder
pub struct EscPDecoder<R> {
    reader: R,
    eof_on_zero: bool,
}

impl<R: io::Read> EscPDecoder<R> {
    /// Create a new instance of the ESC/P reader
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            eof_on_zero: true,
        }
    }

    /// If set to true, return [Command::Eof] when reading returns 0 bytes (normal file)
    pub fn set_eof_on_zero(&mut self, eof_on_zero: bool) {
        self.eof_on_zero = eof_on_zero;
    }
}

/// A single printer command
#[derive(Debug)]
pub enum Command {
    /// End of file
    Eof,
    /// Line Feed (LF, ASCII 10)
    LineFeed,
    /// Form Feed (FF, ASCI 12)
    FormFeed,
    /// Line Feed (CR, ASCII 13)
    CarriageReturn,
    /// Other ASCII characters
    Byte(u8),
    /// Escape sequence
    Esc(Escape),
}

/// Escape sequence
#[derive(Debug)]
#[non_exhaustive]
pub enum Escape {
    /// ESC 3 n
    LineSpacing {
        /// in 1/180th of an inch
        n: u8,
    },
    /// ESC + n
    LineSpacing360 {
        /// in 1/360th of an inch
        n: u8,
    },
    /// ESC $ nl nh
    ///
    /// Set absolute horizontal print position
    XPos {
        /// horizontal position (in defined units)
        ///
        /// The default unit is 1/60th of an inch, updated with the `ESC ( U` command
        n: u16,
    },
    /// Select a bit image
    SelectBitImage {
        /// mode
        m: u8,
        /// number of columns
        n: u16,
        /// data
        data: BitImage,
    },
    /// Unimplemented ESC/P code
    Unknown(Code),
}

impl<R: io::Read> EscPDecoder<R> {
    /// Advance to the next command
    pub fn advance(&mut self) -> io::Result<Command> {
        let mut buf = [0u8; 1];
        let mut cmd_buf = [0u8; 1];
        let mut arg_buf = [0u8; 4];
        let x = loop {
            match self.reader.read(&mut buf) {
                Ok(1..) => break buf[0],
                Ok(0) if self.eof_on_zero => return Ok(Command::Eof),
                Ok(0) => continue,
                Err(e) => {
                    return Err(e);
                }
            }
        };
        match x {
            0x1B => {
                self.reader.read_exact(&mut cmd_buf).expect("ESC <cmd>");
                let cmd = cmd_buf[0];
                match cmd {
                    b'3' => {
                        // Sets the line spacing to n/180 inch
                        self.reader
                            .read_exact(&mut arg_buf[..1])
                            .expect("ESC 3 <arg>");
                        let n = arg_buf[0];
                        Ok(Command::Esc(Escape::LineSpacing { n }))
                    }
                    b'+' => {
                        // Sets the line spacing to n/360 inch
                        self.reader
                            .read_exact(&mut arg_buf[..1])
                            .expect("ESC + <arg>");
                        let n = arg_buf[0];
                        Ok(Command::Esc(Escape::LineSpacing360 { n }))
                    }
                    b'$' => {
                        self.reader
                            .read_exact(&mut arg_buf[..2])
                            .expect("ESC $ <nl> <nh>");
                        let nl = arg_buf[0];
                        let nh = arg_buf[1];
                        let n = ((nh as u16) << 8) + nl as u16;
                        Ok(Command::Esc(Escape::XPos { n }))
                    }
                    b'*' => {
                        self.reader
                            .read_exact(&mut arg_buf[..3])
                            .expect("ESC * <m> <nl> <nh>");
                        let m = arg_buf[0];
                        let nl = arg_buf[1];
                        let nh = arg_buf[2];
                        let n = ((nh as u16) << 8) + nl as u16;
                        let bytes_per_col = bytes_per_col(m).expect("[ESC *] Invalid m");
                        let k = bytes_per_col * n as usize;
                        let mut data = vec![0u8; k];
                        self.reader.read_exact(&mut data).expect("[ESC *] <data>");
                        Ok(Command::Esc(Escape::SelectBitImage {
                            m,
                            n,
                            data: BitImage { raw: data },
                        }))
                    }
                    _ => Ok(Command::Esc(Escape::Unknown(Code(cmd)))),
                }
            }
            b'\n' => Ok(Command::LineFeed),
            b'\r' => Ok(Command::CarriageReturn),
            0x0C => Ok(Command::FormFeed),
            c => Ok(Command::Byte(c)),
        }
    }
}

fn bytes_per_col(m: u8) -> Option<usize> {
    match m {
        0..=7 => Some(1),
        32 | 33 | 38..=40 => Some(3),
        71..=73 => Some(6),
        _ => None,
    }
}

/// An unimplemented escape code
pub struct Code(pub u8);

impl fmt::Debug for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", char::from(self.0))
    }
}

/// An 'inline' bit image (stored in column-order)
pub struct BitImage {
    raw: Vec<u8>,
}

impl BitImage {
    /// Return the bytes as written in the ESC/P stream
    pub fn as_bytes(&self) -> &[u8] {
        self.raw.as_slice()
    }
}

impl fmt::Debug for BitImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BitImage")
            .field("len", &self.raw.len())
            .finish()
    }
}
