//! # Character Maps (CMap)
//!
//! See [Adobe Tech Note #5411] *ToUnicode Mapping File Tutorial*
//!
//! ## More Information
//!
//! Source: <https://github.com/adobe-type-tools/cmap-resources>
//!
//! To learn more about CMap resources, please reference [Adobe Tech Note #5099],
//! *Developing CMap Resources for CID-Keyed Fonts*, and [Adobe Tech Note #5014],
//! *Adobe CMap and CID Font Files Specification*.
//!
//! [Adobe Tech Note #5099]: https://github.com/adobe-type-tools/font-tech-notes/blob/main/pdfs/5099.CMapResources.pdf
//! [Adobe Tech Note #5014]: https://github.com/adobe-type-tools/font-tech-notes/blob/main/pdfs/5014.CIDFont_Spec.pdf
//! [Adobe Tech Note #5411]: https://pdfa.org/norm-refs/5411.ToUnicode.pdf
use std::{fmt, ops::RangeInclusive};

use signum::chsets::encoding::Mapping;

struct BFRange {
    /// Character ID Range
    cids: RangeInclusive<u8>,
    /// Base of mapped unicode code points
    ucs_first: char,
}

struct BFChar {
    /// Character ID (CID)
    cid: u8,
    /// Mapped unicode code-point
    ucs: char,
}

impl BFChar {
    fn write<W: fmt::Write>(&self, out: &mut W) -> fmt::Result {
        let mut buf = [0; 2];
        let slice = self.ucs.encode_utf16(&mut buf);
        write!(out, "<{:02X}> <", self.cid)?;
        for utf16char in slice {
            write!(out, "{:04X}", utf16char)?;
        }
        writeln!(out, ">")?;
        Ok(())
    }
}

/// An in-memory character map
pub struct CMap {
    registry: String,
    ordering: String,
    bfchars: Vec<BFChar>,
    bfranges: Vec<BFRange>,
}

impl BFRange {
    fn write<W: fmt::Write>(&self, out: &mut W) -> fmt::Result {
        let mut buf = [0; 2];
        let slice = self.ucs_first.encode_utf16(&mut buf);
        let start = self.cids.start();
        let end = self.cids.end();
        write!(out, "<{:02X}> <{:02X}> <", start, end)?;
        for utf16char in slice {
            write!(out, "{:04X}", utf16char)?;
        }
        writeln!(out, ">")?;
        Ok(())
    }
}

impl CMap {
    /// Create a new CMap from a mapping
    pub fn new_from_mapping(mapping: &Mapping, name: &str) -> Self {
        let mut bfchars = vec![];
        let mut bfranges = vec![];

        let mut iter = mapping
            .chars
            .iter()
            .copied()
            .enumerate()
            //.filter(|(_, c)| *c != char::REPLACEMENT_CHARACTER)
            .map(|(index, chr)| (index as u8, chr))
            .peekable();
        while let Some((index, chr)) = iter.next() {
            let mut end = index;
            let mut chr_last = u32::from(chr);
            while iter.peek().map(|(_, chr)| u32::from(*chr)) == Some(chr_last + 1) {
                let (next_index, chr_next) = iter.next().unwrap();
                end = next_index;
                chr_last = u32::from(chr_next);
            }
            if end > index {
                bfranges.push(BFRange {
                    cids: index..=end,
                    ucs_first: chr,
                });
            } else {
                bfchars.push(BFChar {
                    cid: index,
                    ucs: chr,
                });
            }
        }

        Self {
            registry: REGISTRY.to_owned(),
            ordering: name.to_owned(),
            bfchars,
            bfranges,
        }
    }

    /// Write the CMap to a formatter
    pub fn write<W: fmt::Write>(&self, out: &mut W, comments: bool) -> fmt::Result {
        // Header
        let registry = self.registry.as_str();
        let ordering = self.ordering.as_str();
        let name = format!("{}-{}-000", registry, ordering);

        if comments {
            writeln!(out, "%!PS-Adobe-3.0 Resource-CMap")?;
            writeln!(out, "%%DocumentNeededResources: ProcSet (CIDInit)")?;
            writeln!(out, "%%IncludeResource: ProcSet (CIDInit)")?;

            writeln!(out, "%%BeginResource: CMap ({name})")?;
            writeln!(out, "%%Title: ({name} {} {} 0)", registry, ordering)?;
            writeln!(out, "%%Version: 1.000")?;
            writeln!(out, "%%EndComments")?;
            writeln!(out)?;
        }
        writeln!(out, "/CIDInit /ProcSet findresource begin")?;
        writeln!(out, "12 dict  begin")?;
        writeln!(out)?;
        writeln!(out, "begincmap")?;
        writeln!(out, "/CIDSystemInfo <<")?;
        writeln!(out, "  /Registry ({})", registry)?;
        writeln!(out, "  /Ordering ({})", ordering)?;
        writeln!(out, "  /Supplement 0")?;
        writeln!(out, ">> def")?;
        writeln!(out)?;
        writeln!(out, "/CMapName /{} def", name)?;
        writeln!(out, "/CMapVersion 1.000 def")?;
        writeln!(out, "/CMapType 2 def")?;
        writeln!(out)?;
        writeln!(out, "1 begincodespacerange")?;
        writeln!(out, "<0000> <FFFF>")?;
        writeln!(out, "endcodespacerange")?;
        if !self.bfchars.is_empty() {
            writeln!(out)?;
            for bfchars in self.bfchars.chunks(100) {
                writeln!(out, "{} beginbfchar", bfchars.len())?;
                for bfchar in bfchars {
                    bfchar.write(out)?;
                }
                writeln!(out, "endbfchar")?;
            }
        }
        if !self.bfchars.is_empty() {
            writeln!(out)?;
            for bfrange in self.bfranges.chunks(100) {
                writeln!(out, "{} beginbfrange", bfrange.len())?;
                for bfrange in bfrange {
                    bfrange.write(out)?;
                }
                writeln!(out, "endbfrange")?;
            }
        }
        writeln!(out)?;
        writeln!(out, "endcmap")?;
        writeln!(out, "CMapName currentdict /CMap defineresource pop")?;
        writeln!(out, "end")?;
        writeln!(out, "end")?;

        if comments {
            writeln!(out)?;
            writeln!(out, "%%EndResource")?;
            writeln!(out, "%%EOF")?;
        }
        Ok(())
    }
}

const REGISTRY: &str = "Signum";

/// Write a character codepoint map (CMap)
pub fn write_cmap<W>(out: &mut W, mapping: &Mapping, name: &str, comments: bool) -> fmt::Result
where
    W: fmt::Write,
{
    let cmap = CMap::new_from_mapping(mapping, name);
    cmap.write(out, comments)?;
    Ok(())
}
