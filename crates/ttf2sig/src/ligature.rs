use ttf_parser::{
    gsub::{LigatureSubstitution, SubstitutionSubtable},
    Face, GlyphId,
};

pub struct LigatureInfo<'a> {
    ligature_substitutions: Vec<LigatureSubstitution<'a>>,
    zwj: Option<GlyphId>,
}

const ZERO_WIDTH_JOINER: char = '\u{200D}';

impl<'a> LigatureInfo<'a> {
    pub fn new(face: &Face<'a>) -> Self {
        Self {
            zwj: face.glyph_index(ZERO_WIDTH_JOINER),
            ligature_substitutions: face
                .tables()
                .gsub
                .map(|gsub| {
                    gsub.lookups
                        .into_iter()
                        .flat_map(|lookup| {
                            lookup
                                .subtables
                                .into_iter::<SubstitutionSubtable>()
                                .filter_map(|subtable| match subtable {
                                    SubstitutionSubtable::Ligature(lig) => Some(lig),
                                    _ => None,
                                })
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    pub fn find(&self, glyphs: &[GlyphId]) -> Option<GlyphId> {
        let (&first, rest) = glyphs.split_first()?;

        for lig in &self.ligature_substitutions {
            if let Some(coverage) = lig.coverage.get(first) {
                let l = lig.ligature_sets.get(coverage)?;
                for ligature in l {
                    if ligature.components.into_iter().eq(rest.iter().copied()) {
                        return Some(ligature.glyph);
                    } else if let Some(zwj) = self.zwj {
                        if ligature
                            .components
                            .into_iter()
                            .eq(rest.iter().copied().flat_map(|i| [zwj, i].into_iter()))
                        {
                            return Some(ligature.glyph);
                        }
                    }
                }
                return None;
            }
        }
        None
    }
}
