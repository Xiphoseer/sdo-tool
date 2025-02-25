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
use std::fmt;

use pdf_create::high::{BFChar, BFRange, ToUnicodeCMap};
use signum::chsets::encoding::Mapping;

const REGISTRY: &str = "Signum";

/// Write a character codepoint map (CMap)
pub fn write_cmap<W>(out: &mut W, mapping: &Mapping, name: &str, comments: bool) -> fmt::Result
where
    W: fmt::Write,
{
    let cmap = new_from_mapping(mapping, name);
    cmap.write(out, comments)?;
    Ok(())
}

/// Create a new CMap from a mapping
pub fn new_from_mapping(mapping: &Mapping, name: &str) -> ToUnicodeCMap {
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
            bfranges.push(BFRange::new(index..=end, chr));
        } else {
            bfchars.push(BFChar::new(index, chr));
        }
    }

    ToUnicodeCMap::new(REGISTRY.to_owned(), name.to_owned(), 0, bfchars, bfranges)
}
