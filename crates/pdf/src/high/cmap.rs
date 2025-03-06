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

use std::{borrow::Cow, fmt, ops::RangeInclusive};

use smallvec::SmallVec;

use crate::{common::StreamMetadata, low, lowering::DebugName};

use super::stream::ToStream;

/// Range of character that map to sequential unicode characters
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BFRange {
    /// Character ID Range
    cids: RangeInclusive<u8>,
    /// Base of mapped unicode code points
    ucs_first: char,
}

impl BFRange {
    /// Create a new [BFRange]
    pub const fn new(cids: RangeInclusive<u8>, ucs_first: char) -> Self {
        Self { cids, ucs_first }
    }

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

/// A simple code to unicode character mapping
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BFChar {
    /// Character ID (CID)
    cid: u8,
    /// Mapped unicode code-points
    ucs: SmallVec<[char; 4]>,
}

impl BFChar {
    /// Create a new [BFChar]
    pub fn new(cid: u8, ucs: char) -> Self {
        Self {
            cid,
            ucs: SmallVec::from_elem(ucs, 1),
        }
    }

    /// Create a new [BFChar]
    pub fn new_slice(cid: u8, ucs: &[char]) -> Self {
        Self {
            cid,
            ucs: SmallVec::from_slice(ucs),
        }
    }

    fn write<W: fmt::Write>(&self, out: &mut W) -> fmt::Result {
        let mut buf = [0; 2];
        write!(out, "<{:02X}> <", self.cid)?;
        for chr in self.ucs.as_slice() {
            let slice = chr.encode_utf16(&mut buf);
            for utf16char in slice {
                write!(out, "{:04X}", utf16char)?;
            }
        }
        writeln!(out, ">")?;
        Ok(())
    }
}

/// An in-memory character map
///
/// This is a special case of the general CMap that always uses a single-byte
/// input encoding and a 16-bit (UTF-16) codespace.
#[derive(Debug, Clone)]
pub struct ToUnicodeCMap {
    registry: String,
    ordering: String,
    supplement: u8,
    bfchars: Vec<BFChar>,
    bfranges: Vec<BFRange>,
}

impl ToUnicodeCMap {
    /// Create a new [ToUnicodeCMap]
    pub fn new(
        registry: String,
        ordering: String,
        supplement: u8,
        bfchars: Vec<BFChar>,
        bfranges: Vec<BFRange>,
    ) -> Self {
        Self {
            registry,
            ordering,
            supplement,
            bfchars,
            bfranges,
        }
    }

    /// Return the name of the CMap as "Registry-Ordering-Supplement"
    pub fn name(&self) -> String {
        let registry = self.registry.as_str();
        let ordering = self.ordering.as_str();
        let supplement = self.supplement;
        format!("{}-{}-{:03}", registry, ordering, supplement)
    }

    /// Write the CMap to a formatter
    pub fn write<W: fmt::Write>(&self, out: &mut W, comments: bool) -> fmt::Result {
        // Header
        let name = self.name();

        if comments {
            writeln!(out, "%!PS-Adobe-3.0 Resource-CMap")?;
            writeln!(out, "%%DocumentNeededResources: ProcSet (CIDInit)")?;
            writeln!(out, "%%IncludeResource: ProcSet (CIDInit)")?;

            writeln!(out, "%%BeginResource: CMap ({name})")?;
            writeln!(
                out,
                "%%Title: ({name} {} {} {})",
                &self.registry, &self.ordering, &self.supplement
            )?;
            writeln!(out, "%%Version: 1.000")?;
            writeln!(out, "%%EndComments")?;
            writeln!(out)?;
        }
        writeln!(out, "/CIDInit /ProcSet findresource begin")?;
        writeln!(out, "12 dict  begin")?;
        writeln!(out)?;
        writeln!(out, "begincmap")?;
        writeln!(out, "/CIDSystemInfo <<")?;
        writeln!(out, "  /Registry ({})", self.registry)?;
        writeln!(out, "  /Ordering ({})", self.ordering)?;
        writeln!(out, "  /Supplement {}", self.supplement)?;
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

impl DebugName for ToUnicodeCMap {
    fn debug_name() -> &'static str {
        "ToUnicode"
    }
}

impl<'a> ToStream<'a> for ToUnicodeCMap {
    type Stream = low::Ascii85Stream<'static>;
    type Error = fmt::Error;

    fn to_stream(&'a self) -> Result<Self::Stream, fmt::Error> {
        let mut out = String::new();
        self.write(&mut out, false)?;
        Ok(low::Ascii85Stream {
            data: Cow::Owned(out.into_bytes()),
            meta: StreamMetadata::None,
        })
    }
}
