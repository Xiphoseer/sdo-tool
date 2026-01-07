/// # Draw bitmap as ascii-art
use std::{fmt, io};

use crate::bits::BitIter;

pub struct BorderDrawing {
    pub left: char,
    pub middle: char,
    pub right: char,
}

pub struct BoxDrawing {
    pub top: BorderDrawing,
    pub left: char,
    pub right: char,
    pub bottom: BorderDrawing,
    pub ink: char,
    pub no_ink: char,
}

const ASCII_BORDER: BorderDrawing = BorderDrawing {
    left: '+',
    middle: '-',
    right: '+',
};
pub const ASCII: &'static BoxDrawing = &BoxDrawing {
    top: ASCII_BORDER,
    left: '|',
    right: '|',
    bottom: ASCII_BORDER,
    ink: '#',
    no_ink: ' ',
};

pub const UNICODE: &'static BoxDrawing = &BoxDrawing {
    top: BorderDrawing {
        left: '╔',
        middle: '═',
        right: '╗',
    },
    left: '║',
    right: '║',
    bottom: BorderDrawing {
        left: '╚',
        middle: '═',
        right: '╝',
    },
    ink: '█',
    no_ink: ' ',
};

/// Draw the packed bitmap using characters
pub fn ascii_art<W: fmt::Write>(
    w: &mut W,
    bitmap: &[u8],
    width: usize,
    invert: bool,
) -> fmt::Result {
    let b: &'static BoxDrawing = UNICODE; // Parameter?
    let height = bitmap.len() * 8 / width;
    let mut iter = BitIter::new(&bitmap);
    w.write_char(b.top.left)?;
    for _ in 0..width {
        w.write_char(b.top.middle)?;
    }
    w.write_char(b.top.right)?;
    w.write_char('\n')?;
    for _ in 0..height {
        w.write_char(b.left)?;
        for _ in 0..width {
            let bit = iter.next().unwrap();
            w.write_char(if bit ^ invert { b.ink } else { b.no_ink })?;
        }
        w.write_char(b.right)?;
        w.write_char('\n')?;
    }
    w.write_char(b.bottom.left)?;
    for _ in 0..width {
        w.write_char(b.bottom.middle)?;
    }
    w.write_char(b.bottom.right)?;
    w.write_char('\n')?;
    Ok(())
}

/// Write a packed bitmap as a PBM file
pub fn pbm_to_io_writer<W: io::Write>(
    w: &mut W,
    bitmap: &[u8],
    width: usize,
    invert: bool,
    dbl: bool,
) -> io::Result<()> {
    let height = bitmap.len() * 8 / width;
    let mut iter = BitIter::new(&bitmap);
    let mut s = String::with_capacity(width);
    writeln!(w, "P1 {} {}", width, if dbl { height * 2 } else { height })?;
    for _ in 0..height {
        s.clear();
        for _ in 0..width {
            let bit = iter.next().unwrap();
            s.push(if bit ^ invert { '1' } else { '0' });
        }
        writeln!(w, "{s}")?;
        if dbl {
            writeln!(w, "{s}")?;
        }
    }
    Ok(())
}
