use std::{
    borrow::Cow,
    collections::BTreeMap,
    io::{self, Write},
};

use ccitt_t4_t6::g42d::encode::Encoder;
use pdf_create::{
    common::{
        BaseEncoding, Dict, Encoding, FontDescriptor, FontFlags, Matrix, PdfString, Point,
        Rectangle, SparseSet, StreamMetadata,
    },
    high::{Ascii85Stream, Font, ResourceIndex},
    write::{PdfName, PdfNameBuf, PdfNameStr},
};
use sdo_ps::dvips::CacheDevice;
use signum::chsets::{
    cache::ChsetCache,
    editor::ESet,
    encoding::Mapping,
    printer::{PSet, PSetChar, PrinterKind},
    UseTable, UseTableVec,
};

use crate::cmap::write_cmap;

#[rustfmt::skip]
pub const DEFAULT_NAMES: [&str; 128] = [
    "NUL",         "Zparenleft", "Zparenright", "Zslash",     "Zasterisk", "Zzero",     "Zone",        "Ztwo",
    "Zthree",      "Zfour",      "Zfive",       "Zsix",       "Zseven",    "Zeight",    "Znine",       "zparenleft",
    // 16
    "zparenright", "zslash",     "zasterisk",   "zzero",      "zone",      "ztwo",      "zthree",      "zfour",
    "zfive",       "zsix",       "zseven",      "zeight",     "znine",     "zplus",     "zminus",      "zperiod",
    // 32
    "section",     "exclam",     "quotedbl",    "numbersign", "dollar",    "percent",   "ampersand",   "quotesingle",
    "parenleft",   "parenright", "asterisk",    "plus",       "comma",     "hyphen",     "period",      "slash",
    // 48
    "zero",        "one",        "two",         "three",      "four",      "five",      "six",         "seven",
    "eight",       "nine",       "colon",       "semicolon",  "less",      "equal",     "greater",     "question",
    // 64
    "udieresis",   "A",          "B",           "C",          "D",         "E",         "F",           "G",
    "H",           "I",          "J",           "K",          "L",         "M",         "N",           "O",
    // 80
    "P",           "Q",          "R",           "S",          "T",         "U",         "V",           "W",
    "X",           "Y",          "Z",           "odieresis",  "Udieresis", "adieresis", "asciicircum", "underscore",
    // 96
    "grave",       "a",          "b",           "c",          "d",         "e",         "f",           "g",
    "h",           "i",          "j",           "k",          "l",         "m",         "n",           "o",
    // 112
    "p",           "q",          "r",           "s",          "t",         "u",         "v",           "w",
    "x",           "y",          "z",           "Odieresis",  "bar",       "Adieresis", "asciitilde",  "germandbls",
];

/// Charcodes of all characters that have a different name compared to the `WinAnsiEncoding`
pub const DIFFERENCES: &[u8] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 64, 91, 92, 93, 123, 125, 127,
];

pub struct FontMetrics {
    pub baseline: u8,
    pub pixels_per_inch_x: u32,
    pub pixels_per_inch_y: u32,
    pub pixels_per_pdfunit_x: u32,
    pub pixels_per_pdfunit_y: u32,
    pub fontunits_per_pixel_x: u32,
    pub fontunits_per_pixel_y: u32,
}

impl From<PrinterKind> for FontMetrics {
    fn from(pk: PrinterKind) -> Self {
        let pdfunits_per_inch = 72;
        let fontunits_per_inch = pdfunits_per_inch * 1000;
        let (pixels_per_inch_x, pixels_per_inch_y) = pk.resolution();

        let pixels_per_pdfunit_x = pixels_per_inch_x / pdfunits_per_inch;
        let pixels_per_pdfunit_y = pixels_per_inch_y / pdfunits_per_inch;

        let fontunits_per_pixel_x = fontunits_per_inch / pixels_per_inch_x;
        let fontunits_per_pixel_y = fontunits_per_inch / pixels_per_inch_y;
        Self {
            baseline: pk.baseline(),

            pixels_per_inch_x,
            pixels_per_inch_y,

            pixels_per_pdfunit_x,
            pixels_per_pdfunit_y,

            fontunits_per_pixel_x,
            fontunits_per_pixel_y,
        }
    }
}

pub fn write_char_stream<W: Write>(
    w: &mut W,
    pchar: &PSetChar,
    dx: u32,
    font_metrics: &FontMetrics,
) -> io::Result<()> {
    // This is all in pixels
    let hb = pchar.hbounds();
    let ur_x = (pchar.width as usize) * 8 - hb.max_tail;
    let ll_x = hb.max_lead;
    let box_width = ur_x - ll_x;
    let box_height = pchar.height as usize;
    let mut encoder = Encoder::new(box_width, pchar.bitmap);
    encoder.skip_lead = hb.max_lead;
    encoder.skip_tail = hb.max_tail;
    let buf = encoder.encode();

    // The default font size
    let font_size = 10;

    // This is in pixels
    let top = font_metrics.baseline as i8;
    let ur_y = top - pchar.top as i8;
    let ll_y = ur_y - pchar.height as i8;

    let fpx = font_metrics.fontunits_per_pixel_x as i32 / font_size;
    let fpy = font_metrics.fontunits_per_pixel_y as i32 / font_size;

    // This is all in font units
    let cd = CacheDevice {
        w_x: dx as i16,
        w_y: 0,
        ll_x: ll_x as i32 * fpx,
        ll_y: ll_y as i32 * fpy,
        ur_x: ur_x as i32 * fpx,
        ur_y: ur_y as i32 * fpy,
    };
    writeln!(
        w,
        "{} {} {} {} {} {} d1",
        cd.w_x, cd.w_y, cd.ll_x, cd.ll_y, cd.ur_x, cd.ur_y
    )?;

    let gc_w = box_width as i32 * fpx;
    let gc_h = (box_height as i32) * fpy;
    let gc_x = ll_x as i32 * fpx;
    let gc_y = ll_y as i32 * fpy;
    writeln!(w, "{} 0 0 {} {} {} cm", gc_w, gc_h, gc_x, gc_y)?;
    writeln!(w, "BI")?;
    writeln!(w, "  /IM true")?;
    writeln!(w, "  /W {}", box_width)?;
    writeln!(w, "  /H {}", box_height)?;
    writeln!(w, "  /BPC 1")?;
    writeln!(w, "  /D[0 1]")?;
    writeln!(w, "  /F/CCF")?;
    //writeln!(w, "  /CS/CalGray")?;
    writeln!(w, "  /DP<</K -1/Columns {}>>", box_width)?;
    writeln!(w, "ID")?;

    w.write_all(&buf)?;

    writeln!(w, "EI")?;
    Ok(())
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Font variants
pub enum FontVariant {
    /// Regular
    Regular,
    /// Italic
    Italic,
    /// Bold
    Bold,
    /// Italic & Bold
    ItalicBold,
}

pub struct Type3FontVariant<'a> {
    /// The name of the font
    pub name: Cow<'a, PdfNameStr>,
    /// The matrix to map glyph space into text space
    ///
    /// this is useful for creating an italic font
    pub font_matrix: Matrix<f32>,
    /// Font characteristics
    pub font_descriptor: FontDescriptor<'a>,
}

pub struct Type3FontFamily<'a> {
    pub font_variants: BTreeMap<FontVariant, Type3FontVariant<'a>>,
    /// The largest boundig box that fits all glyphs
    pub font_bbox: Rectangle<i32>,
    /// The first used char key
    pub first_char: u8,
    /// The last used char key
    pub last_char: u8,
    /// Dict of char names to drawing procedures
    pub char_procs: Dict<Ascii85Stream<'a>>,
    /// Dict of encoding value to char names
    pub encoding: Encoding<'a>,
    /// Width of every char between first and last
    pub widths: Vec<u32>,
    /// ToUnicode CMap stream
    pub to_unicode: Option<Ascii85Stream<'a>>,
}

pub fn type3_font_family<'a>(
    efont: Option<&'a ESet>,
    pfont: &'a PSet,
    use_table: &UseTable,
    mappings: Option<&Mapping>,
    name: &'a str,
) -> Option<Type3FontFamily<'a>> {
    let font_metrics = FontMetrics::from(pfont.pk);
    let font_matrix = Matrix::scale(0.001, -0.001);
    let font_matrix_italic = font_matrix * Matrix::shear_x(-0.25);

    let (first_char, last_char) = use_table.first_last()?;
    let capacity = (last_char - first_char + 1) as usize;
    let mut widths = Vec::with_capacity(capacity);
    let mut procs: Vec<(&str, Vec<u8>)> = Vec::with_capacity(capacity);

    let mut max_width = 0;

    let mut max_bottom = 0;
    let mut min_top = pfont.pk.line_height();

    let font_size = 10;

    for cval in first_char..=last_char {
        let cvu = cval as usize;
        let ewidth = if let Some(efont) = efont {
            efont.chars[cvu].width
        } else {
            todo!("missing character #{} in editor font", cvu);
        };
        if ewidth > 0 && use_table.chars[cvu] > 0 {
            let width = u32::from(ewidth) * (800 / font_size);
            widths.push(width);
            max_width = max_width.max(width as i32);

            let pchar = &pfont.chars[cvu];
            if pchar.width > 0 {
                let mut cproc = Vec::new();
                write_char_stream(&mut cproc, pchar, width, &font_metrics).unwrap();
                procs.push((DEFAULT_NAMES[cvu], cproc));
                max_bottom = max_bottom.max(pchar.top as u32 + pchar.height as u32);
                min_top = min_top.min(pchar.top as u32);
            } else {
                // FIXME: empty glyph for non-printable characters?
            }
        } else {
            widths.push(0);
        }
    }

    let gchar = &pfont.chars[b'g' as usize];
    let gchar_descent = gchar.top as i32 + gchar.height as i32;
    let descent = pfont.pk.baseline() as i32 - gchar_descent;

    let achar = &pfont.chars[b'A' as usize];
    let achar_ascent = achar.top as i32;
    let ascent = pfont.pk.baseline() as i32 - achar_ascent;

    assert!(min_top <= max_bottom);
    //let max_height = max_bottom - min_top;

    let ll_y = pfont.pk.baseline() as i32 - max_bottom as i32;
    let ur_y = pfont.pk.baseline() as i32 - min_top as i32;

    let fpy = font_metrics.fontunits_per_pixel_y as i32;

    let font_bbox = Rectangle {
        ll: Point {
            x: 0,
            y: ll_y * fpy,
        },
        ur: Point {
            x: max_width,
            y: ur_y * fpy,
        },
    };

    let mut char_procs = Dict::new();
    for (name, cproc) in procs {
        char_procs.insert(
            String::from(name),
            Ascii85Stream {
                data: Cow::Owned(cproc.to_owned()),
                meta: StreamMetadata::None,
            },
        );
    }

    let mut differences = SparseSet::with_size(256);
    for cval in DIFFERENCES {
        let i = *cval as usize;
        if use_table.chars[i] > 0 {
            // skip unused chars
            differences[i] = Some(PdfName::new(DEFAULT_NAMES[i]));
        }
    }

    // FIXME: update to include `encode_byte` cases
    let to_unicode = mappings.map(|mapping| {
        let mut out = String::new();
        write_cmap(&mut out, mapping, name).unwrap();
        Ascii85Stream {
            data: Cow::Owned(out.into_bytes()),
            meta: StreamMetadata::None,
        }
    });

    let mut font_variants = BTreeMap::new();
    font_variants.insert(FontVariant::Regular, {
        let font_name = PdfNameBuf::new(format!("{}-Regular", name));
        let font_descriptor = FontDescriptor {
            font_name: Cow::Owned(font_name.clone()),
            font_family: PdfString::new(name),
            font_stretch: None,
            font_weight: None,
            flags: FontFlags::SYMBOLIC,
            font_bbox: Some(font_bbox),
            italic_angle: 0.0,
            ascent: Some((ascent * fpy) / 18),
            descent: Some((descent * fpy) / 18),
            leading: None,
            cap_height: None,
            x_height: None,
            stem_v: None,
            stem_h: None,
        };
        Type3FontVariant {
            name: Cow::Owned(font_name),
            font_matrix,
            font_descriptor,
        }
    });

    font_variants.insert(FontVariant::Italic, {
        let font_name = PdfNameBuf::new(format!("{}-Italic", name));
        let font_descriptor = FontDescriptor {
            font_name: Cow::Owned(font_name.clone()),
            font_family: PdfString::new(name),
            font_stretch: None,
            font_weight: None,
            flags: FontFlags::SYMBOLIC,
            font_bbox: Some(font_bbox),
            italic_angle: -22.5,
            ascent: Some((ascent * fpy) / 18),
            descent: Some((descent * fpy) / 18),
            leading: None,
            cap_height: None,
            x_height: None,
            stem_v: None,
            stem_h: None,
        };
        Type3FontVariant {
            name: Cow::Owned(font_name),
            font_matrix: font_matrix_italic,
            font_descriptor,
        }
    });

    Some(Type3FontFamily {
        font_bbox,
        first_char,
        last_char,
        char_procs,
        encoding: Encoding {
            base_encoding: Some(BaseEncoding::WinAnsiEncoding),
            differences: Some(differences),
        },
        widths,
        to_unicode,
        font_variants,
    })
}

pub struct FontInfo {
    widths: Vec<u32>,
    first_char: u8,
    index: usize,
}

impl FontInfo {
    pub fn width(&self, cval: u8) -> u32 {
        assert!(cval < 128);
        let fc = self.first_char;
        let wi = (cval - fc) as usize;
        self.widths[wi]
    }
}

pub struct Fonts {
    info: Vec<Option<FontInfo>>,
    base: usize,
}

pub enum MakeFontsErr {}

impl Fonts {
    pub fn index<'a>(&self, info: &FontInfo, variant: FontVariant) -> ResourceIndex<Font<'a>> {
        // FIXME: times two is for regular and italic
        let off = match variant {
            FontVariant::Regular => 0,
            FontVariant::Italic => 1,
            FontVariant::Bold => todo!(),
            FontVariant::ItalicBold => todo!(),
        };
        ResourceIndex::new(self.base + info.index * 2 + off)
    }

    pub fn get(&self, fc_index: usize) -> Option<&FontInfo> {
        self.info[fc_index].as_ref()
    }

    pub fn new(fonts_capacity: usize, base: usize) -> Self {
        Fonts {
            info: Vec::with_capacity(fonts_capacity),
            base,
        }
    }

    pub fn make_fonts<'a>(
        &mut self,
        fc: &'a ChsetCache,
        use_table_vec: UseTableVec,
        pk: PrinterKind,
    ) -> Vec<Type3FontFamily<'a>> {
        let chsets = fc.chsets();
        let mut result = Vec::with_capacity(chsets.len());
        for (index, cs) in chsets.iter().enumerate() {
            let use_table = &use_table_vec.csets[index];

            let info = cs.printer(pk).and_then(|pfont| {
                let efont = cs.e24();
                let mappings = cs.map();
                type3_font_family(efont, pfont, use_table, mappings, cs.name()).map(|font| {
                    let info = FontInfo {
                        widths: font.widths.clone(),
                        first_char: font.first_char,
                        index: result.len(),
                    };
                    result.push(font);
                    info
                })
            });
            self.info.push(info);
        }
        result
    }
}
