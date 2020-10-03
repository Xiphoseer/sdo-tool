use std::{
    borrow::Cow,
    io::{self, Write},
};

use ccitt_t4_t6::g42d::encode::Encoder;
use pdf_create::{
    common::{BaseEncoding, Dict, Encoding, Matrix, Point, Rectangle, SparseSet},
    high::{CharProc, Type3Font},
    write::PdfName,
};
use sdo::font::{
    dvips::CacheDevice,
    editor::ESet,
    printer::{PSet, PSetChar, PrinterKind},
    UseTable,
};

#[rustfmt::skip]
pub const DEFAULT_NAMES: [&str; 128] = [
    "NUL",         "Zparenleft", "Zparenright", "Zslash",     "Zasterisk", "Zzero",     "Zone",        "Ztwo",
    "Zthree",      "Zfour",      "Zfive",       "Zsix",       "Zseven",    "Zeight",    "Znine",       "zparenleft",
    // 16
    "zparenright", "zslash",     "zasterisk",   "zzero",      "zone",      "ztwo",      "zthree",      "zfour",
    "zfive",       "zsix",       "zseven",      "zeight",     "znine",     "zplus",     "zminus",      "zperiod",
    // 32
    "section",     "exclam",     "quotedbl",    "numbersign", "dollar",    "percent",   "ampersand",   "quotesingle",
    "parenleft",   "parenright", "asterisk",    "plus",       "comma",     "minus",     "period",      "slash",
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
    "x",           "y",          "z",           "Adieresis",  "bar",       "Odieresis", "asciitilde",  "germandbls",
];

/// Charcodes of all characters that have a different name compared to the `WinAnsiEncoding`
pub const DIFFERENCES: &[u8] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 45, 64, 91, 92, 93, 123, 125, 127,
];

pub fn write_char_stream<W: Write>(
    w: &mut W,
    pchar: &PSetChar,
    dx: u32,
    pk: PrinterKind,
) -> io::Result<()> {
    let hb = pchar.hbounds();
    let ur_x = (pchar.width as usize) * 8 - hb.max_tail;
    let ll_x = hb.max_lead;
    let box_width = ur_x - ll_x;
    let box_height = pchar.height as usize;
    let mut encoder = Encoder::new(box_width, &pchar.bitmap);
    encoder.skip_lead = hb.max_lead;
    encoder.skip_tail = hb.max_tail;
    let buf = encoder.encode();

    let top = pk.baseline();
    let ur_y = top - (pchar.top as i16);
    let ll_y = ur_y - (pchar.height as i16);

    let cd = CacheDevice {
        w_x: dx as i16,
        w_y: 0,
        ll_x: ll_x as i16,
        ll_y,
        ur_x: ur_x as i16,
        ur_y,
    };
    writeln!(
        w,
        "{} {} {} {} {} {} d1",
        cd.w_x, cd.w_y, cd.ll_x, cd.ll_y, cd.ur_x, cd.ur_y
    )?;
    writeln!(w, "0.01 0 0 0.01 0 0 cm")?;
    writeln!(w, "q")?;

    let gc_w = box_width * 100;
    let gc_h = box_height * 100;
    let gc_y = ll_y * 100; // + 10;
    let gc_x = ll_x * 100; // + 10;
    writeln!(w, "{} 0 0 {} {} {} cm", gc_w, gc_h, gc_x, gc_y)?;
    writeln!(w, "BI")?;
    writeln!(w, "  /IM true")?;
    writeln!(w, "  /W {}", box_width)?;
    writeln!(w, "  /H {}", box_height)?;
    writeln!(w, "  /BPC 1")?;
    writeln!(w, "  /D[0 1]")?;
    writeln!(w, "  /F/CCF")?;
    writeln!(w, "  /DP<</K -1/Columns {}>>", box_width)?;
    writeln!(w, "ID")?;

    w.write_all(&buf)?;

    writeln!(w, "EI")?;
    writeln!(w, "Q")?;
    Ok(())
}

pub fn type3_font<'a>(
    efont: Option<&'a ESet>,
    pfont: &'a PSet,
    pk: PrinterKind,
    use_table: &UseTable,
    name: Option<&'a str>,
) -> Option<Type3Font<'a>> {
    let font_bbox = Rectangle {
        ll: Point::default(),
        ur: Point { x: 1, y: -1 },
    };
    let font_matrix = Matrix::scale(pk.scale(), -pk.scale());

    let (first_char, last_char) = use_table.first_last()?;
    let capacity = (last_char - first_char + 1) as usize;
    let mut widths = Vec::with_capacity(capacity);
    let mut procs: Vec<(&str, Vec<u8>)> = Vec::with_capacity(capacity);

    for cval in first_char..=last_char {
        let cvu = cval as usize;
        let ewidth = if let Some(efont) = efont {
            efont.chars[cvu].width
        } else {
            todo!();
        };
        if ewidth > 0 && use_table.chars[cvu] > 0 {
            let width = pk.scale_x(ewidth.into());
            widths.push(width);

            let pchar = &pfont.chars[cvu];
            if pchar.width > 0 {
                let mut cproc = Vec::new();
                write_char_stream(&mut cproc, pchar, width, pk).unwrap();
                procs.push((DEFAULT_NAMES[cvu], cproc));
            } else {
                // FIXME: empty glyph for non-printable character?
            }
        } else {
            widths.push(0);
        }
    }

    let mut char_procs = Dict::new();
    for (name, cproc) in procs {
        char_procs.insert(String::from(name), CharProc(Cow::Owned(cproc.to_owned())));
    }

    let mut differences = SparseSet::with_size(256);
    for cval in DIFFERENCES {
        let i = *cval as usize;
        if use_table.chars[i] > 0 {
            // skip unused chars
            differences[i] = Some(PdfName(DEFAULT_NAMES[i]));
        }
    }

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
        to_unicode: (),
    })
}
