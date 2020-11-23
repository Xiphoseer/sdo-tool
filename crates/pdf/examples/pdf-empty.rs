use chrono::Local;
use color_eyre::eyre;
use pdf_create::{
    common::Point,
    common::{PdfString, Rectangle},
    high::{Handle, Page, Resources},
};

pub fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    // Create a new handle
    let mut doc = Handle::new();

    // Set some metadata
    doc.info.author = Some(PdfString::new("Xiphoseer"));
    doc.info.creator = Some(PdfString::new("SIGNUM (c) 1986-93 F. Schmerbeck"));
    doc.info.producer = Some(PdfString::new("Signum! Document Toolbox"));
    doc.info.title = Some(PdfString::new("EMPTY.SDO"));
    doc.info.mod_date = Some(Local::now());
    doc.info.creation_date = Some(Local::now());

    // Create a page
    let page = Page {
        media_box: Rectangle {
            ll: Point { x: 0, y: 0 },
            ur: Point { x: 592, y: 842 },
        },
        resources: Resources::default(),
        contents: Vec::new(),
    };

    // Add the page to the document
    doc.pages.push(page);

    // Write the PDF to the console
    let stdout = std::io::stdout();
    let mut stdolock = stdout.lock();
    doc.write(&mut stdolock)?;

    Ok(())
}
