use std::{
    borrow::Cow,
    io::{self, Write},
};

use ccitt_t4_t6::g42d::encode::Encoder;
use pdf_create::{
    common::{
        BaseEncoding, Dict, Encoding, FontDescriptor, FontFlags, Matrix, PdfString, Point,
        Rectangle, SparseSet, StreamMetadata,
    },
    high::{Ascii85Stream, Font, Type3Font},
    write::PdfName,
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

/*/// Charcodes of all characters that have a different name compared to the `WinAnsiEncoding`
pub const DIFFERENCES: &[u8] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 64, 91, 92, 93, 123, 125, 127,
];*/

pub const MAPPED: &[u8] = &[32, 64, 91, 92, 93, 123, 125, 127];

pub fn encode_byte(cval: u8) -> u8 {
    match cval {
        // Turn a space into a `WinAnsiEncoding` `section`
        32 => 0o247,  // section
        64 => 0o374,  // udieresis
        91 => 0o366,  // odieresis
        92 => 0o334,  // Udieresis
        93 => 0o344,  // adieresis
        123 => 0o326, // Odieresis
        125 => 0o304, // Adieresis
        127 => 0o337, // germandbls
        _ => cval,
    }
}

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

const COLOR_SPACE: &str = "[/CalGray<</WhitePoint[0.9505 1.0000 1.0890]>>]";

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
    writeln!(w, "  /CS{}", COLOR_SPACE)?;
    writeln!(w, "  /DP<</K -1/Columns {}>>", box_width)?;
    writeln!(w, "ID")?;

    w.write_all(&buf)?;

    writeln!(w, "EI")?;
    Ok(())
}

pub fn type3_font<'a>(
    efont: Option<&'a ESet>,
    pfont: &'a PSet,
    use_table: &UseTable,
    mappings: Option<&Mapping>,
    name: Option<&'a str>,
) -> Option<Type3Font<'a>> {
    let font_metrics = FontMetrics::from(pfont.pk);
    let font_matrix = Matrix::scale(0.001, -0.001);

    let (first_char, last_char) = use_table.first_last()?;
    let glyph_count = (last_char - first_char + 1) as usize;
    
    // Fixup mapped chars
    let mut last_code = last_char;
    for &code in MAPPED {
        if use_table.chars[code as usize] > 0 {
            let coded = encode_byte(code);
            if coded > last_code {
                last_code = coded;
            }
        }
    }

    let capacity = (last_code - first_char + 1) as usize;
    let mut widths = Vec::with_capacity(capacity);
    let mut procs: Vec<(&str, Vec<u8>)> = Vec::with_capacity(glyph_count);

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
            let coded = encode_byte(cval);
            let index = (coded - first_char) as usize;
            while widths.len() <= index {
                widths.push(0);
            }
            widths[index] = width;
            
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
    for i in 0..32 {
        if use_table.chars[i] > 0 {
            // skip unused chars
            differences[i] = Some(PdfName(DEFAULT_NAMES[i]));
        }
    }

    let font_descriptor = name.map(|name| FontDescriptor {
        font_name: PdfName(name),
        font_family: PdfString::new(name),
        font_stretch: None,
        font_weight: None,
        flags: FontFlags::SYMBOLIC,
        font_bbox: Some(font_bbox),
        italic_angle: 0,
        ascent: Some((ascent * fpy) / 18),
        descent: Some((descent * fpy) / 18),
        leading: None,
        cap_height: None,
        x_height: None,
        stem_v: None,
        stem_h: None,
    });

    // FIXME: update to include `encode_byte` cases
    let to_unicode = mappings.map(|mapping| {
        let mut out = String::new();
        write_cmap(&mut out, mapping, name.unwrap_or("UNKNOWN")).unwrap();
        Ascii85Stream {
            data: Cow::Owned(out.into_bytes()),
            meta: StreamMetadata::None,
        }
    });

    Some(Type3Font {
        name: name.map(PdfName),
        font_bbox,
        font_matrix,
        first_char,
        last_char: last_code,
        char_procs,
        font_descriptor,
        encoding: Encoding {
            base_encoding: Some(BaseEncoding::WinAnsiEncoding),
            differences: Some(differences),
        },
        widths,
        to_unicode,
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
        let idx = encode_byte(cval);
        let wi = (idx - fc) as usize;
        self.widths[wi]
    }
}

pub struct Fonts {
    info: Vec<Option<FontInfo>>,
    base: usize,
}

pub enum MakeFontsErr {}

impl Fonts {
    pub fn index(&self, info: &FontInfo) -> usize {
        self.base + info.index
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
    ) -> Vec<Font<'a>> {
        let chsets = fc.chsets();
        let mut result = Vec::with_capacity(chsets.len());
        for (index, cs) in chsets.iter().enumerate() {
            let use_table = &use_table_vec.csets[index];

            if let Some(pfont) = cs.printer(pk) {
                // FIXME: FontDescriptor

                let efont = cs.e24();
                let mappings = cs.map();
                if let Some(font) = type3_font(efont, pfont, use_table, mappings, Some(cs.name())) {
                    let info = FontInfo {
                        widths: font.widths.clone(),
                        first_char: font.first_char,
                        index: result.len(),
                    };
                    self.info.push(Some(info));
                    result.push(Font::Type3(font));
                    continue;
                }
            }
            self.info.push(None);
        }
        result
    }
}
