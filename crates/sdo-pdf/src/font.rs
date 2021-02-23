use std::{
    borrow::Cow,
    io::{self, Write},
};

use ccitt_t4_t6::g42d::encode::Encoder;
use pdf_create::{
    common::{BaseEncoding, Dict, Encoding, Matrix, Point, Rectangle, SparseSet},
    high::{Ascii85Stream, Type3Font},
    write::PdfName,
};
use sdo_ps::dvips::CacheDevice;
use signum::chsets::{
    editor::ESet,
    encoding::Mapping,
    printer::{PSet, PSetChar, PrinterKind},
    UseTable,
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
    pub baseline: i32,
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
    let mut encoder = Encoder::new(box_width, &pchar.bitmap);
    encoder.skip_lead = hb.max_lead;
    encoder.skip_tail = hb.max_tail;
    let buf = encoder.encode();

    // This is all in font units
    let top = font_metrics.baseline;
    let ur_y = top - (pchar.top as i32);
    let ll_y = ur_y - pchar.height as i32;

    let cd = CacheDevice {
        w_x: dx as i16,
        w_y: 0,
        ll_x: ll_x as i32 * font_metrics.fontunits_per_pixel_x as i32,
        ll_y: ll_y * font_metrics.fontunits_per_pixel_y as i32,
        ur_x: ur_x as i32 * font_metrics.fontunits_per_pixel_x as i32,
        ur_y: ur_y * font_metrics.fontunits_per_pixel_y as i32,
    };
    writeln!(
        w,
        "{} {} {} {} {} {} d1",
        cd.w_x, cd.w_y, cd.ll_x, cd.ll_y, cd.ur_x, cd.ur_y
    )?;

    let fpx = font_metrics.fontunits_per_pixel_x;
    let fpy = font_metrics.fontunits_per_pixel_y;

    let gc_w = box_width as i32 * fpx as i32;
    let gc_h = box_height as i32 * fpy as i32;
    let gc_x = ll_x as i32 * fpx as i32;
    let gc_y = ll_y as i32 * fpy as i32;
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
    let capacity = (last_char - first_char + 1) as usize;
    let mut widths = Vec::with_capacity(capacity);
    let mut procs: Vec<(&str, Vec<u8>)> = Vec::with_capacity(capacity);

    let mut max_width = 0;
    let mut max_height = 0;

    for cval in first_char..=last_char {
        let cvu = cval as usize;
        let ewidth = if let Some(efont) = efont {
            efont.chars[cvu].width
        } else {
            todo!("missing character #{} in editor font", cvu);
        };
        if ewidth > 0 && use_table.chars[cvu] > 0 {
            let width = u32::from(ewidth) * 800;
            widths.push(width);
            max_width = max_width.max(width as i32);

            let pchar = &pfont.chars[cvu];
            if pchar.width > 0 {
                let mut cproc = Vec::new();
                write_char_stream(&mut cproc, pchar, width, &font_metrics).unwrap();
                procs.push((DEFAULT_NAMES[cvu], cproc));
                max_height = max_height.max(pchar.height as i32 * 200);
            } else {
                // FIXME: empty glyph for non-printable characters?
            }
        } else {
            widths.push(0);
        }
    }

    let font_bbox = Rectangle {
        ll: Point { x: 0, y: 0 },
        ur: Point {
            x: max_width,
            y: max_height,
        },
    };

    let mut char_procs = Dict::new();
    for (name, cproc) in procs {
        char_procs.insert(
            String::from(name),
            Ascii85Stream(Cow::Owned(cproc.to_owned())),
        );
    }

    let mut differences = SparseSet::with_size(256);
    for cval in DIFFERENCES {
        let i = *cval as usize;
        if use_table.chars[i] > 0 {
            // skip unused chars
            differences[i] = Some(PdfName(DEFAULT_NAMES[i]));
        }
    }

    let to_unicode = mappings.map(|mapping| {
        let mut out = String::new();
        write_cmap(&mut out, mapping, name.unwrap_or("UNKNOWN")).unwrap();
        Ascii85Stream(Cow::Owned(out.into_bytes()))
    });

    Some(Type3Font {
        name: name.map(|name| PdfName(name)),
        font_bbox,
        font_matrix,
        first_char,
        last_char,
        char_procs,
        encoding: Encoding {
            base_encoding: Some(BaseEncoding::WinAnsiEncoding),
            differences: Some(differences),
        },
        widths,
        to_unicode,
    })
}
