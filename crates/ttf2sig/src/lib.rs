mod kerning;
mod ligature;

pub use kerning::KerningInfo;
pub use ligature::LigatureInfo;
use ttf_parser::{Face, GlyphId};

pub fn glyph_index_vec(face: &Face<'_>, lig: &[char]) -> Option<Vec<GlyphId>> {
    let mut v = Vec::new();
    for &code_point in lig {
        v.push(face.glyph_index(code_point)?);
    }
    Some(v)
}
