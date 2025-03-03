//! # Fonts

use std::{
    borrow::Cow,
    io::{self, Write},
    str::FromStr,
};

use ccitt_t4_t6::g42d::encode::Encoder;
use pdf_create::{
    common::{
        BaseEncoding, Dict, Encoding, FontDescriptor, FontFlags, Matrix, PdfString, Point,
        Rectangle, SparseSet, StreamMetadata,
    },
    high::{
        Ascii85Stream, DictResource, Font, GlobalResource, Res, Resource, ToUnicode, Type3Font,
    },
    write::{PdfName, PdfNameBuf},
};
use sdo_ps::dvips::CacheDevice;
use signum::{
    chsets::{
        cache::{CSet, ChsetCache, DocumentFontCacheInfo, FontCacheInfo},
        editor::ESet,
        printer::{PSet, PSetChar, PrinterKind},
        UseMatrix, UseTable, UseTableVec,
    },
    docs::GenerationContext,
    util::Buf,
};

use crate::cmap;

/// Names for all signum glyph positions, e.g. `Zfive`
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

/// Font metrics
pub struct FontMetrics {
    baseline: i32,
    // pixels_per_inch_x: u32,
    // pixels_per_inch_y: u32,
    // pixels_per_pdfunit_x: u32,
    // pixels_per_pdfunit_y: u32,
    fontunits_per_pixel_x: u32,
    fontunits_per_pixel_y: u32,
}

const PDFUNITS_PER_INCH: u32 = 72;
const FONTUNITS_PER_INCH: u32 = PDFUNITS_PER_INCH * 1000;

impl From<PrinterKind> for FontMetrics {
    fn from(pk: PrinterKind) -> Self {
        let pixels_per_inch = pk.resolution();

        // let pixels_per_pdfunit_x = pixels_per_inch_x / pdfunits_per_inch;
        // let pixels_per_pdfunit_y = pixels_per_inch_y / pdfunits_per_inch;

        let fontunits_per_pixel_x = FONTUNITS_PER_INCH / pixels_per_inch.x;
        let fontunits_per_pixel_y = FONTUNITS_PER_INCH / pixels_per_inch.y;
        Self {
            baseline: pk.baseline(),

            // pixels_per_inch_x,
            // pixels_per_inch_y,

            // pixels_per_pdfunit_x,
            // pixels_per_pdfunit_y,
            fontunits_per_pixel_x,
            fontunits_per_pixel_y,
        }
    }
}

pub(crate) const DEFAULT_FONT_SIZE: i32 = 10;

const _COLOR_SPACE: &str = "[/CalGray<</WhitePoint[0.9505 1.0000 1.0890]>>]";
const COLOR_SPACE: &str = "/G";

/// Write a printer character to the stream
pub fn write_char_stream<W: Write>(
    w: &mut W,
    pchar: &PSetChar,
    dx: u32,
    font_metrics: &FontMetrics,
) -> io::Result<()> {
    // This is all in pixels
    let hb = pchar.hbounds();
    let right_x = (pchar.width as usize) * 8 - hb.max_tail;
    let left_x = hb.max_lead;
    let box_width = right_x - left_x;
    let box_height = pchar.height as usize;
    let mut encoder = Encoder::new(box_width, &pchar.bitmap);
    encoder.skip_lead = hb.max_lead;
    encoder.skip_tail = hb.max_tail;
    let buf = encoder.encode();

    // The default font size
    let font_size = DEFAULT_FONT_SIZE;

    // This is in pixels
    let top = font_metrics.baseline;
    let upper_y = top - (pchar.top as i32);
    let lower_y = upper_y - pchar.height as i32;

    let fpx = font_metrics.fontunits_per_pixel_x as i32 / font_size;
    let fpy = font_metrics.fontunits_per_pixel_y as i32 / font_size;

    // This is all in font units
    let cd = CacheDevice {
        w_x: dx as i16,
        w_y: 0,
        ll_x: left_x as i32 * fpx,
        ll_y: lower_y * fpy,
        ur_x: right_x as i32 * fpx,
        ur_y: upper_y * fpy,
    };
    writeln!(
        w,
        "{} {} {} {} {} {} d1",
        cd.w_x, cd.w_y, cd.ll_x, cd.ll_y, cd.ur_x, cd.ur_y
    )?;

    let gc_w = box_width as i32 * fpx;
    let gc_h = box_height as i32 * fpy;
    let gc_x = left_x as i32 * fpx;
    let gc_y = lower_y * fpy;
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

/// Number of font-units (1/72000 of an inch) per horizontal signum unit (1/90 of an inch)
pub(crate) const FONTUNITS_PER_SIGNUM_X: u32 = 800;

const EMPTY_GLYPH_PROC: &[u8] = b"0 0 0 0 0 0 d1";

/// Calculate all the glyph widths
pub fn glyph_widths(efont: &ESet<'_>) -> Vec<u32> {
    let mut widths = Vec::with_capacity(128);
    for i in 0..128 {
        let ewidth = efont.chars[i].width;
        let font_size = DEFAULT_FONT_SIZE as u32;
        let width = u32::from(ewidth) * (FONTUNITS_PER_SIGNUM_X / font_size);
        widths.push(width);
    }
    widths
}

/// Create a type 3 font
pub fn type3_font<'a>(
    widths: &[u32], // in font-units, see `glyph_widths`
    pfont: &PSet,
    use_table: &UseTable,
    to_unicode: Option<Resource<ToUnicode>>,
    name: &str,
) -> Option<Type3Font<'a>> {
    let font_metrics = FontMetrics::from(pfont.pk);
    let font_matrix = Matrix::scale(0.001, 0.001);

    let (first_char, last_char) = use_table.first_last()?;
    let capacity = (last_char - first_char + 1) as usize;
    let mut procs: Vec<(&str, Vec<u8>)> = Vec::with_capacity(capacity);

    let mut max_width = 0;
    let mut max_above_baseline = 0;
    let mut max_below_baseline = 0;

    let font_size = DEFAULT_FONT_SIZE as u32;

    let fpy = font_metrics.fontunits_per_pixel_y as i32 / font_size as i32;

    for cval in first_char..=last_char {
        let cvu = cval as usize;
        let width = widths[cvu];
        let num_uses = use_table.chars[cvu];
        let pchar = &pfont.chars[cvu];

        // calculate font metrics
        let sig_origin_y = font_metrics.baseline;
        let sig_upper_y = sig_origin_y - pchar.top as i32;
        let sig_lower_y = sig_upper_y - pchar.height as i32;
        max_above_baseline = max_above_baseline.max(sig_upper_y * fpy);
        max_below_baseline = max_below_baseline.min(sig_lower_y * fpy);

        if width > 0 && num_uses > 0 {
            max_width = max_width.max(width as i32);

            if pchar.width > 0 {
                let mut cproc = Vec::new();
                write_char_stream(&mut cproc, pchar, width, &font_metrics).unwrap();
                procs.push((DEFAULT_NAMES[cvu], cproc));
            } else {
                // FIXME: empty glyph for non-printable characters?
                log::warn!(
                    "Missing spacer glyph {} in {:?} [used {} time(s)], inserting empty glyph",
                    cvu,
                    name,
                    num_uses
                );
                procs.push((DEFAULT_NAMES[cvu], EMPTY_GLYPH_PROC.to_vec()));
            }
        } else if num_uses > 0 {
            log::warn!(
                "Empty zero-advance glyph {} in {:?} [used {} time(s)], inserting empty glyph",
                cvu,
                name,
                num_uses
            );
            procs.push((DEFAULT_NAMES[cvu], EMPTY_GLYPH_PROC.to_vec()));
        }
    }

    // FIXME: this works best in evice => why?
    let ascent = pfont.pk.ascent() as i32 * fpy * 2 / 3;
    let descent = -(pfont.pk.descent() as i32) * fpy / 4;

    let font_bbox = Rectangle {
        ll: Point {
            x: 0,
            y: max_below_baseline,
        },
        ur: Point {
            x: max_width,
            y: max_above_baseline,
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
            differences[i] = Some(PdfName(DEFAULT_NAMES[i]));
        }
    }

    let font_descriptor = Some(FontDescriptor {
        font_name: PdfNameBuf::new(name),
        font_family: PdfString::from_str(name).unwrap(),
        font_stretch: None,
        font_weight: None,
        flags: FontFlags::SYMBOLIC,
        font_bbox: Some(font_bbox),
        italic_angle: 0,
        ascent: Some(ascent),
        descent: Some(descent),
        leading: None,
        cap_height: None,
        x_height: None,
        stem_v: None,
        stem_h: None,
    });

    Some(Type3Font {
        name: Some(PdfNameBuf::new(name)),
        font_bbox,
        font_matrix,
        first_char,
        last_char,
        char_procs,
        font_descriptor,
        encoding: Encoding {
            base_encoding: Some(BaseEncoding::WinAnsiEncoding),
            differences: Some(differences),
        },
        widths: widths[(first_char as usize)..=(last_char as usize)].to_vec(),
        to_unicode,
    })
}

/// Information on one font
pub struct FontInfo {
    /// The widths of each glyph (int fontunits, i.e. 1/72000 in)
    widths: Vec<u32>,
    /// Index within the PDF document of the font resource
    index: Option<GlobalResource<Font<'static>>>,
    /// Index within the PDF document of the font resource (bold variant)
    index_bold: Option<GlobalResource<Font<'static>>>,
}

impl FontInfo {
    /// Get the width of the character in this font
    pub fn width(&self, cval: u8) -> u32 {
        assert!(cval < 128);
        self.widths[cval as usize]
    }
}

/// Information on multiple fonts
pub struct Fonts {
    info: Vec<Option<FontInfo>>,
}

/// Error when creating fonts
pub enum MakeFontsErr {}

impl Fonts {
    /// Get the font info by index in the font cache
    pub fn get(&self, fc_index: usize) -> Option<&FontInfo> {
        self.info[fc_index].as_ref()
    }

    /// Get the info by [FontCacheInfo]
    pub fn info<'a>(&'a self, fci: &FontCacheInfo) -> Option<&'a FontInfo> {
        fci.index().and_then(|fc_index| self.get(fc_index))
    }

    /// Create a new instance
    pub fn new(fonts_capacity: usize) -> Self {
        Fonts {
            info: Vec::with_capacity(fonts_capacity),
        }
    }

    /// For all fonts in a font cache, add them to the resources
    pub fn make_fonts<'a>(
        &mut self,
        fc: &'a ChsetCache,
        res: &mut Res<'a>,
        use_table_vec: UseTableVec,
        use_table_vec_bold: UseTableVec,
        pk: PrinterKind,
    ) {
        let chsets = fc.chsets();
        for (index, cs) in chsets.iter().enumerate() {
            let use_table = &use_table_vec.csets[index];
            let use_table_bold = &use_table_vec_bold.csets[index];

            let info = make_font(res, pk, cs, use_table, use_table_bold);
            self.info.push(info);
        }
    }
}

fn make_font<'a>(
    res: &mut Res<'a>,
    pk: PrinterKind,
    cs: &'a CSet,
    use_table: &UseTable,
    use_table_bold: &UseTable,
) -> Option<FontInfo> {
    let pfont = cs.printer(pk)?;
    let efont = cs.e24().expect("editor font required"); // FIXME: widths?
    let widths = glyph_widths(efont);

    let mappings = cs.map();
    let to_unicode = mappings
        .map(|mapping| cmap::new_from_mapping(mapping, cs.name()))
        .map(|to_unicode| res.push_to_unicode(to_unicode))
        .map(Resource::from);

    let font_regular = type3_font(&widths, pfont, use_table, to_unicode.clone(), cs.name())
        .map(|f| res.push_font(Font::Type3(f)));
    let font_bold = type3_font(
        &widths,
        &pset_bold(pfont),
        use_table_bold,
        to_unicode,
        &format!("{}-Bold", cs.name()),
    )
    .map(|f| res.push_font(Font::Type3(f)));
    if font_regular.is_none() && font_bold.is_none() {
        return None;
    }
    let info = FontInfo {
        widths,
        index: font_regular,
        index_bold: font_bold,
    };
    Some(info)
}

fn pset_bold(pfont: &PSet<'_>) -> PSet<'static> {
    PSet {
        pk: pfont.pk,
        header: Buf(&[]),
        chars: pfont.chars.iter().map(PSetChar::bold_normal).collect(),
    }
}

/// The names used for the charsets in a font
pub const FONTS: [&str; 8] = ["C0", "C1", "C2", "C3", "C4", "C5", "C6", "C7"];

/// The names used for the bold charsets in a font
pub const FONTS_BOLD: [&str; 8] = ["B0", "B1", "B2", "B3", "B4", "B5", "B6", "B7"];

impl Fonts {
    /// Prepare the font dictionary
    pub fn font_dict<'a>(
        &'a self,
        print: &DocumentFontCacheInfo,
    ) -> (DictResource<Font<'static>>, [Option<&'a FontInfo>; 8]) {
        let mut infos = [None; 8];
        let mut dict = DictResource::new();
        for (cset, info) in print
            .font_cache_info()
            .iter()
            .enumerate()
            .filter_map(|(cset, fci)| self.info(fci).map(|info| (cset, info)))
        {
            if let Some(index) = info.index {
                dict.insert(FONTS[cset].to_owned(), Resource::from(index));
            }
            if let Some(index) = info.index_bold {
                dict.insert(FONTS_BOLD[cset].to_owned(), Resource::from(index));
            }
            infos[cset] = Some(info);
        }
        (dict, infos)
    }
}

/// Prepare the PDF fonts
pub fn prepare_pdf_fonts<'f, GC: GenerationContext>(
    res: &mut Res<'f>,
    gc: &GC,
    fc: &'f ChsetCache,
    pk: PrinterKind,
) -> Fonts {
    let pages = gc.text_pages();
    let dfci = gc.fonts();
    let use_table_vec = {
        let mut v = UseTableVec::new();
        let use_matrix_regular = UseMatrix::of_matching(pages, |k| !k.style.is_bold());
        v.append(dfci, use_matrix_regular);
        v
    };
    let use_table_vec_bold = {
        let mut v = UseTableVec::new();
        let use_matrix_bold = UseMatrix::of_matching(pages, |k| k.style.is_bold());
        v.append(dfci, use_matrix_bold);
        v
    };

    let mut font_info = Fonts::new(8);
    font_info.make_fonts(fc, res, use_table_vec, use_table_vec_bold, pk);
    font_info
}
