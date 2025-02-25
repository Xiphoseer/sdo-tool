use crate::{
    common::{Dict, Encoding, FontDescriptor, Matrix, ObjRef, Point, Rectangle},
    low,
    lowering::{DebugName, LowerBox, Lowerable},
    util::NextId,
    write::PdfName,
};

use super::{Ascii85Stream, Resource, ToUnicodeCMap};

#[derive(Debug, Clone)]
/// A type 3 font
pub struct Type3Font<'a> {
    /// The name of the font
    pub name: Option<PdfName<'a>>,
    /// The largest boundig box that fits all glyphs
    pub font_bbox: Rectangle<i32>,
    /// Font characteristics
    pub font_descriptor: Option<FontDescriptor<'a>>,
    /// The matrix to map glyph space into text space
    pub font_matrix: Matrix<f32>,
    /// The first used char key
    pub first_char: u8,
    /// The last used char key
    pub last_char: u8,
    /// Dict of char names to drawing procedures
    pub char_procs: Dict<Ascii85Stream<'a>>,
    /// Dict of encoding value to char names
    pub encoding: Encoding<'a>,
    /// Width of every char between first and last (in fontunits, i.e. 1/72000 in)
    pub widths: Vec<u32>,
    /// ToUnicode CMap stream
    pub to_unicode: Option<Resource<Ascii85Stream<'a>>>,
}

impl Default for Type3Font<'_> {
    fn default() -> Self {
        Self {
            font_bbox: Rectangle {
                ll: Point::default(),
                ur: Point::default(),
            },
            name: None,
            font_matrix: Matrix::default_glyph(),
            font_descriptor: None,
            first_char: 0,
            last_char: 255,
            char_procs: Dict::new(),
            encoding: Encoding {
                base_encoding: None,
                differences: None,
            },
            widths: vec![],
            to_unicode: None,
        }
    }
}

#[derive(Debug, Clone)]
/// A Font resource
pub enum Font<'a> {
    /// A type 3 font i.e. arbitrary glyph drawings
    Type3(Type3Font<'a>),
}

impl DebugName for Font<'_> {
    fn debug_name() -> &'static str {
        "Font"
    }
}

impl<'a> Lowerable<'a> for Font<'a> {
    type Lower = low::Font<'a>;
    type Ctx = LowerFontCtx<'a>;

    fn lower(&'a self, ctx: &mut Self::Ctx, id_gen: &mut NextId) -> Self::Lower {
        lower_font(self, ctx, id_gen)
    }
}

#[allow(dead_code)]
pub(crate) struct LowerFontCtx<'a> {
    text_streams: LowerBox<'a, Ascii85Stream<'a>>,
    encodings: LowerBox<'a, Encoding<'a>>,
    to_unicode: LowerBox<'a, ToUnicodeCMap>,
}

impl<'a> LowerFontCtx<'a> {
    pub(crate) fn new(
        char_procs: &'a [Ascii85Stream<'a>],
        encodings: &'a [Encoding<'a>],
        to_unicode: &'a [ToUnicodeCMap],
    ) -> Self {
        Self {
            text_streams: LowerBox::new(char_procs),
            encodings: LowerBox::new(encodings),
            to_unicode: LowerBox::new(to_unicode),
        }
    }

    pub(crate) fn text_stream_values(
        &self,
    ) -> impl Iterator<Item = (ObjRef, &'a Ascii85Stream<'a>)> + '_ {
        self.text_streams.store_values()
    }

    pub(crate) fn to_unicode_values(
        &self,
    ) -> impl Iterator<Item = (ObjRef, &'a ToUnicodeCMap)> + '_ {
        self.to_unicode.store_values()
    }

    pub(crate) fn encoding_values(&self) -> impl Iterator<Item = (ObjRef, &'a Encoding<'a>)> + '_ {
        self.encodings.store_values()
    }
}

fn lower_font<'a>(
    font: &'a Font<'a>,
    ctx: &mut LowerFontCtx<'a>,
    id_gen: &mut NextId,
) -> low::Font<'a> {
    match font {
        Font::Type3(font) => {
            let char_procs = font
                .char_procs
                .iter()
                .map(|(key, proc)| {
                    let re = ctx.text_streams.put(proc, id_gen);
                    (key.clone(), re)
                })
                .collect();
            let to_unicode = font
                .to_unicode
                .as_ref()
                .map(|res| ctx.text_streams.map_ref(res, id_gen));
            low::Font::Type3(low::Type3Font {
                name: font.name,
                font_bbox: font.font_bbox,
                font_descriptor: font.font_descriptor.clone(),
                font_matrix: font.font_matrix,
                first_char: font.first_char,
                last_char: font.last_char,
                encoding: low::Resource::Immediate(font.encoding.clone()),
                char_procs,
                widths: &font.widths,
                to_unicode,
            })
        }
    }
}
