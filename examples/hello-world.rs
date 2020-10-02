use std::{collections::BTreeMap, path::PathBuf};

use structopt::StructOpt;

use color_eyre::eyre::{self, eyre};
use pdf::primitive::PdfString;
use pdf_create::{
    common::{BaseEncoding, Dict, Encoding, Matrix, Point, Rectangle, SparseSet},
    high::{CharProc, Font, Handle, Page, Resource, Resources, Type3Font},
    write::PdfName,
};
use sdo::font::{editor::parse_eset, printer::parse_ls30, printer::PrinterKind, FontKind};
use sdo::nom::Finish;
use sdo_pdf::font::{write_char_stream, DEFAULT_NAMES, DIFFERENCES};

#[derive(StructOpt)]
struct Options {
    font: PathBuf,
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
        a: 0.24,
        b: 0.0,
        c: 0.0,
        d: -0.24,
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

    let pd = FontKind::Printer(PrinterKind::Laser30);

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
        //"q 0.1 0 0 0.1 0 0 cm",
        //"/R7 gs",
        "0 g",
        //"q",
        //"10 0 0 10 0 0 cm",
        "BT",
        "/C0 1 Tf",
        "1 0 0 -1 91.9199 759.82 Tm",
        "[(Hello)-7000(J@rgen)]TJ",
        "211.68 654.72 Td",
        "(1)Tj",
        "ET",
        //"Q",
        //"Q",
    ];
    let mut contents = String::new();
    for line in lines.iter() {
        contents.push_str(line);
        contents.push('\n');
    }

    let page = Page {
        media_box: Rectangle::a4_media_box(),
        resources,
        contents,
    };
    doc.pages.push(page);

    doc.write(&mut stdolock)?;

    Ok(())
}
