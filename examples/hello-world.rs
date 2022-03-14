use std::{collections::BTreeMap, path::PathBuf};

use structopt::StructOpt;

use color_eyre::eyre::{self, eyre};
use pdf_create::{
    common::Rectangle,
    high::{Font, Handle, Page, Resource, Resources},
};
use sdo_pdf::font::type3_font;
use signum::chsets::{editor::parse_eset, printer::parse_ls30, UseTable};
use signum::nom::Finish;

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

    let mut doc = Handle::new();

    doc.meta.author = vec!["Xiphoseer".to_string()];
    doc.meta.creator = Some("SIGNUM (c) 1986-93 F. Schmerbeck".to_string());
    doc.meta.producer = "Signum! Document Toolbox".to_string();
    doc.meta.title = Some("EMPTY.SDO".to_string());

    let use_table = UseTable::from("HelloJ@rgen!1");

    let mut fonts = BTreeMap::new();
    if let Some(font) = type3_font(Some(&efont), &pfont, &use_table, None, None) {
        doc.res.fonts.push(Font::Type3(font));
        fonts.insert(String::from("C0"), Resource::Global { index: 0 });
    }

    doc.res.font_dicts.push(fonts);

    let resources = Resources {
        fonts: Resource::Global { index: 0 },
        ..Default::default()
    };

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
        contents: contents.into_bytes(),
    };
    doc.pages.push(page);

    let stdout = std::io::stdout();
    let mut stdolock = stdout.lock();
    doc.write(&mut stdolock)?;

    Ok(())
}
