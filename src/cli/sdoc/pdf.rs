use std::{fs::File, io::BufWriter, path::Path};

use color_eyre::eyre;
use pdf::primitive::PdfString;
use pdf_create::high::Handle;

use super::Document;

pub fn process_doc<'a>(doc: &'a Document) -> Handle<'a> {
    let mut hnd = Handle::new();

    if let Some(author) = &doc.opt.author {
        let author = author.to_owned().into_bytes();
        hnd.info.author = Some(PdfString::new(author));
    }
    let creator = String::from("SIGNUM (c) 1986-93 F. Schmerbeck").into_bytes();
    hnd.info.creator = Some(PdfString::new(creator));
    let producer = String::from("Signum! Document Toolbox").into_bytes();
    hnd.info.producer = Some(PdfString::new(producer));
    // FIXME: string encoding
    let title = doc
        .file
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned()
        .into_bytes();
    hnd.info.title = Some(PdfString::new(title));

    hnd
}

pub fn output_pdf(doc: &Document) -> eyre::Result<()> {
    let hnd = process_doc(doc);

    if doc.opt.out == Path::new("-") {
        println!("----------------------------- PDF -----------------------------");
        let stdout = std::io::stdout();
        let mut stdolock = stdout.lock();
        hnd.write(&mut stdolock)?;
        println!("---------------------------------------------------------------");
        Ok(())
    } else {
        let file = doc.file.file_stem().unwrap();
        let out = {
            let mut buf = doc.opt.out.join(file);
            buf.set_extension("ps");
            buf
        };
        let out_file = File::create(&out)?;
        let mut out_buf = BufWriter::new(out_file);
        print!("Writing `{}` ...", out.display());
        hnd.write(&mut out_buf)?;
        println!(" Done!");
        Ok(())
    }
}
