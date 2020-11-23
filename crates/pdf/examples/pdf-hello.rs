use std::collections::BTreeMap;

use color_eyre::eyre;
use pdf_create::{
    common::Point,
    common::{PdfString, Rectangle},
    high::{Font, Handle, Page, Resource, Resources, Type3Font},
};

pub fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let mut doc = Handle::new();

    doc.info.author = Some(PdfString::new("Xiphoseer"));
    doc.info.creator = Some(PdfString::new("SIGNUM (c) 1986-93 F. Schmerbeck"));
    doc.info.producer = Some(PdfString::new("Signum! Document Toolbox"));
    doc.info.title = Some(PdfString::new("EMPTY.SDO"));

    // FIXME: Add some glyphs/char procs here
    doc.res.fonts.push(Font::Type3(Type3Font::default()));
    doc.res.fonts.push(Font::Type3(Type3Font::default()));

    let mut fonts = BTreeMap::new();
    fonts.insert(String::from("CSET0"), Resource::Global { index: 0 });
    fonts.insert(String::from("CSET1"), Resource::Global { index: 1 });

    doc.res.font_dicts.push(fonts);

    let mut resources = Resources::default();
    resources.fonts = Resource::Global { index: 0 };

    let lines = [
        "q 0.1 0 0 0.1 0 0 cm",
        "/R7 gs",
        "0 g",
        "q",
        "10 0 0 10 0 0 cm BT",
        "/R16 0.24 Tf",
        "1 0 0 -1 91.9199 759.82 Tm",
        "[(Hello)-13000(World!)]TJ",
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
            ll: Point { x: 0, y: 0 },
            ur: Point { x: 592, y: 842 },
        },
        resources,
        contents: contents.into_bytes(),
    };
    doc.pages.push(page);

    let stdout = std::io::stdout();
    let mut stdolock = stdout.lock();
    doc.write(&mut stdolock)?;

    Ok(())
}
