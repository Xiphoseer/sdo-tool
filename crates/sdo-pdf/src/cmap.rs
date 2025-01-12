//! # Character Maps (CMap)
use std::fmt;

use signum::chsets::encoding::Mapping;

/// Write a character codepoint map (CMap)
pub fn write_cmap<W>(out: &mut W, mapping: &Mapping, name: &str) -> fmt::Result
where
    W: fmt::Write,
{
    writeln!(out, "/CIDInit /ProcSet findresource begin")?;
    writeln!(out, "12 dict  begin")?;
    writeln!(out, "begincmap")?;
    writeln!(out, "/CIDSystemInfo")?;
    writeln!(out, "<< /Registry (Signum)")?;
    writeln!(out, "/Ordering (UCS)")?;
    writeln!(out, "/Supplement 0")?;
    writeln!(out, ">> def")?;
    writeln!(out, "/CMapName /Signum-{} def", name)?;
    writeln!(out, "/CMapType 2 def")?;
    writeln!(out, "1 begincodespacerange")?;
    writeln!(out, "<00> <7F>")?;
    writeln!(out, "endcodespacerange")?;
    writeln!(out, "128 beginbfchar")?;
    for (index, chr) in mapping.chars.iter().cloned().enumerate() {
        let mut buf = [0; 2];
        let slice = chr.encode_utf16(&mut buf);
        write!(out, "<{:02X}> <", index)?;
        for utf16char in slice {
            write!(out, "{:04X}", utf16char)?;
        }
        writeln!(out, ">")?;
    }
    writeln!(out, "endbfchar")?;
    writeln!(out, "endcmap")?;
    writeln!(out, "CMapName currentdict /CMap defineresource pop")?;
    writeln!(out, "end")?;
    writeln!(out, "end")?;
    Ok(())
}
