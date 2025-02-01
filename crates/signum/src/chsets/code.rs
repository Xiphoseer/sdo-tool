//! # Rust code generation

use std::fmt;

use super::encoding::Mapping;

/// Write a mapping as rust code
pub fn write_map<W: fmt::Write>(mapping: &Mapping, out: &mut W, ident: &str) -> fmt::Result {
    writeln!(out, "#[rustfmt::skip]")?;
    writeln!(out, "pub const {ident}: [char; 128] = [")?;
    for chars in mapping.chars.chunks(16) {
        write!(out, "    ")?;
        for char in chars {
            /*if *char == '\0' {
                write!(out, "NUL, ")?;
            } else if *char == char::REPLACEMENT_CHARACTER {
                write!(out, "REP, ")?;
            } else {
                write!(out, "{:?}, ", char)?;
            }*/
            write!(out, "'\\u{{{:04X}}}', ", u32::from(*char))?;
        }
        writeln!(out,)?;
    }
    writeln!(out, "];")?;
    Ok(())
}
