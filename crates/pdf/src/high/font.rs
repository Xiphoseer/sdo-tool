use crate::{
    common::{Dict, Encoding, FontDescriptor, Matrix, Point, Rectangle},
    write::PdfName,
};

use super::{Ascii85Stream, Resource};

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
