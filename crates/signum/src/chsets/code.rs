//! # Rust code generation

use std::fmt;

use super::encoding::{Mapping, MappingImpl};

/// Write a mapping as rust code
pub fn write_map<W: fmt::Write>(mapping: &Mapping, out: &mut W, ident: &str) -> fmt::Result {
    writeln!(out, "#[rustfmt::skip]")?;
    if let Some(chars) = mapping.as_char_array() {
        writeln!(out, "pub static {ident}: ::signum::chsets::encoding::Mapping = signum::chsets::encoding::Mapping::new_static(&[")?;
        for row in chars.chunks(16) {
            write!(out, "    ")?;
            for char in row {
                write!(out, "'\\u{{{:04X}}}', ", u32::from(*char))?;
            }
            writeln!(out,)?;
        }
        writeln!(out, "]);")?;
    } else if let MappingImpl::Dynamic(chars) = &mapping.0 {
        writeln!(out, "pub static {ident}: ::signum::chsets::encoding::Mapping = signum::chsets::encoding::Mapping::new_static_slices(&[")?;
        for row in chars.chunks(16) {
            write!(out, "    ")?;
            for slice in row {
                assert!(slice.len() <= 2);
                write!(out, "&[")?;
                for char in slice {
                    write!(out, "'\\u{{{:04X}}}', ", u32::from(*char))?;
                }
                write!(out, "], ")?;
            }
            writeln!(out,)?;
        }
        writeln!(out, "]);")?;
    } else {
        unimplemented!()
    }
    Ok(())
}
