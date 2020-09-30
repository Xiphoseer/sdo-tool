use std::{
    collections::BTreeMap,
    io::{self, Write},
    path::PathBuf,
};

use structopt::StructOpt;

use ccitt_t4_t6::g42d::encode::Encoder;
use color_eyre::eyre::{self, eyre};
use pdf::primitive::PdfString;
use pdf_create::{
    common::BaseEncoding,
    common::Dict,
    common::Encoding,
    common::Matrix,
    common::Point,
    common::Rectangle,
    common::SparseSet,
    high::{CharProc, Font, Handle, Page, Resource, Resources, Type3Font},
    write::PdfName,
};
use sdo::font::{
    dvips::CacheDevice, editor::parse_eset, pdf::DEFAULT_NAMES, pdf::DIFFERENCES,
    printer::parse_ls30, printer::PSetChar, printer::PrintDriver,
};
use sdo::nom::Finish;

#[derive(StructOpt)]
struct Options {
    font: PathBuf,
}

fn write_char_stream<W: Write>(
    w: &mut W,
    pchar: &PSetChar,
    dx: u32,
    pd: PrintDriver,
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

pub fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let opt = Options::from_args();

    let pfont_path = opt.font;
    let pfont_buffer = std::fs::read(&pfont_path)?;
    let (_, pfont) = parse_ls30(&pfont_buffer)
        .finish() //
        .map_err(|_| {
            eyre!(
                "Could not parse printer font file: {}",
                pfont_path.display()
            )
        })?;

    let efont_path = pfont_path.with_extension("E24");
    let efont_buffer = std::fs::read(&efont_path)?;
    let (_, efont) = parse_eset(&efont_buffer)
        .finish()
        .map_err(|_| eyre!("Could not parse editor font file: {}", efont_path.display()))?;

    let stdout = std::io::stdout();
    let mut stdolock = stdout.lock();

    let mut doc = Handle::new();

    let author = String::from("Xiphoseer").into_bytes();
    doc.info.author = Some(PdfString::new(author));
    let creator = String::from("SIGNUM (c) 1986-93 F. Schmerbeck").into_bytes();
    doc.info.creator = Some(PdfString::new(creator));
    let producer = String::from("Signum! Document Toolbox").into_bytes();
    doc.info.producer = Some(PdfString::new(producer));
    let title = String::from("EMPTY.SDO").into_bytes();
    doc.info.title = Some(PdfString::new(title));

    let font_bbox = Rectangle {
        ll: Point::default(),
        ur: Point { x: 1, y: -1 },
    };
    let font_matrix = Matrix {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: -1.0,
        e: 0.0,
        f: 0.0,
    };

    let mut differences = SparseSet::with_size(256);
    for cval in DIFFERENCES {
        let i = *cval as usize;
        differences[i] = Some(PdfName(DEFAULT_NAMES[i]));
    }

    let first_char: u8 = 1;
    let last_char: u8 = 127;
    let capacity = (last_char - first_char + 1) as usize;
    let mut widths = Vec::with_capacity(capacity);
    let mut procs: Vec<(&str, Vec<u8>)> = Vec::with_capacity(capacity);

    let pd = PrintDriver::Laser30;

    for cval in first_char..=last_char {
        let echar = &efont.chars[cval as usize];
        if echar.width > 0 {
            let width = pd.scale_x(echar.width.into());
            widths.push(width);

            let pchar = &pfont.chars[cval as usize];
            if pchar.width > 0 {
                let mut cproc = Vec::new();
                write_char_stream(&mut cproc, pchar, width, pd).unwrap();
                procs.push((DEFAULT_NAMES[cval as usize], cproc));
            } else {
                // FIXME: empty glyph for non-printable character?
            }
        } else {
            widths.push(0);
        }
    }

    let mut char_procs = Dict::new();
    for (name, cproc) in &procs {
        char_procs.insert(String::from(*name), CharProc(cproc));
    }

    doc.res.fonts.push(Font::Type3(Type3Font {
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
    }));

    let mut fonts = BTreeMap::new();
    fonts.insert(String::from("C0"), Resource::Global { index: 0 });

    doc.res.font_dicts.push(fonts);

    let mut resources = Resources::default();
    resources.fonts = Resource::Global { index: 0 };

    let lines = [
        "q 0.1 0 0 0.1 0 0 cm",
        "/R7 gs",
        "0 g",
        "q",
        "10 0 0 10 0 0 cm BT",
        "/C0 0.24 Tf",
        "1 0 0 -1 91.9199 759.82 Tm",
        "[(Hello)-13000(J@rgen)]TJ",
        "211.68 654.72 Td",
        "(1)Tj",
        "ET",
        "Q",
        "Q",
    ];
    let mut contents = String::new();
    for line in lines.iter() {
        contents.push_str(line);
        contents.push('\n');
    }

    let page = Page {
        media_box: Rectangle {
            // A4
            ll: Point { x: 0, y: 0 },
            ur: Point { x: 592, y: 842 },
        },
        resources,
        contents,
    };
    doc.pages.push(page);

    doc.write(&mut stdolock)?;

    Ok(())
}
