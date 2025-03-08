use ttf_parser::{
    gpos::{PairAdjustment, PositioningSubtable, ValueRecord},
    Face, GlyphId,
};

#[derive(Debug)]
pub struct KerningInfo<'a> {
    pair_adjustments: Vec<PairAdjustment<'a>>,
}

impl<'a> KerningInfo<'a> {
    pub fn new(face: &Face<'a>) -> Self {
        Self {
            pair_adjustments: face
                .tables()
                .gpos
                .map(|gpos| {
                    gpos.lookups
                        .into_iter()
                        .flat_map(|lookup| {
                            lookup
                                .subtables
                                .into_iter::<PositioningSubtable>()
                                .filter_map(|subtable| match subtable {
                                    PositioningSubtable::Pair(pair) => Some(pair),
                                    _ => None,
                                })
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    pub fn find(
        &self,
        first: GlyphId,
        second: GlyphId,
    ) -> Option<(ValueRecord<'a>, ValueRecord<'a>)> {
        for p in &self.pair_adjustments {
            if let Some(c) = p.coverage().get(first) {
                match p {
                    PairAdjustment::Format1 { coverage: _, sets } => {
                        let set = sets.get(c).expect("coverage");
                        return set.get(second);
                    }
                    PairAdjustment::Format2 {
                        coverage: _,
                        classes: _,
                        matrix: _,
                    } => unimplemented!(),
                }
            }
        }
        None
    }
}
