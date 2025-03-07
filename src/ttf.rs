use ttf_parser::{
    gsub::{LigatureSubstitution, SubstitutionSubtable},
    Face, GlyphId,
};

pub struct LigatureInfo<'a> {
    ligature_substitutions: Vec<LigatureSubstitution<'a>>,
}

impl<'a> LigatureInfo<'a> {
    pub fn new(face: &Face<'a>) -> Self {
        Self {
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
                    }
                }
                return None;
            }
        }
        None
    }
}

pub fn glyph_index_vec(face: &Face<'_>, lig: &[char]) -> Option<Vec<GlyphId>> {
    let mut v = Vec::new();
    for &code_point in lig {
        v.push(face.glyph_index(code_point)?);
    }
    Some(v)
}
