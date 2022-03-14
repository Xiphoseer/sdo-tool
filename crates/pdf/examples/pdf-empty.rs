use color_eyre::eyre;
use pdf_create::{
    common::Point,
    common::Rectangle,
    high::{Handle, Page, Resources},
};

pub fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    // Create a new handle
    let mut doc = Handle::new();

    // Set some metadata
    doc.meta.author = vec!["Xiphoseer".to_string()];
    doc.meta.creator = Some("SIGNUM (c) 1986-93 F. Schmerbeck".to_string());
    doc.meta.producer = "Signum! Document Toolbox".to_string();
    doc.meta.title = Some("EMPTY.SDO".to_string());

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
