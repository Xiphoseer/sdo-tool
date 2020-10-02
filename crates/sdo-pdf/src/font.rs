use std::io::{self, Write};

use ccitt_t4_t6::g42d::encode::Encoder;
use sdo::font::{dvips::CacheDevice, printer::PSetChar, FontKind};

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
    26, 27, 28, 29, 30, 31, 32, 64, 91, 92, 93, 123, 125, 127,
];

pub fn write_char_stream<W: Write>(
    w: &mut W,
    pchar: &PSetChar,
    dx: u32,
    pd: FontKind,
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

    let top = pd.baseline();
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
