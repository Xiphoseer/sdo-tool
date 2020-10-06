use std::{collections::BTreeMap, fs::File, io::BufWriter, path::Path};

use color_eyre::eyre::{self, eyre};
use pdf::primitive::PdfString;
use pdf_create::{
    chrono::Local,
    common::Rectangle,
    encoding::pdf_doc_encode,
    high::{Font, Handle, Page, Resource, Resources},
};
use sdo::font::FontKind;
use sdo_pdf::{font::type3_font, sdoc::Contents};

use super::Document;

pub fn process_doc<'a>(doc: &'a Document) -> eyre::Result<Handle<'a>> {
    let mut hnd = Handle::new();

    let meta = &doc.opt.meta()?;

    if let Some(author) = &meta.author {
        let author = pdf_doc_encode(author)?;
        hnd.info.author = Some(PdfString::new(author));
    }
    if let Some(subject) = &meta.subject {
        let subject = pdf_doc_encode(subject)?;
        hnd.info.subject = Some(PdfString::new(subject));
    }
    if let Some(title) = &meta.title {
        let title = pdf_doc_encode(title)?;
        hnd.info.title = Some(PdfString::new(title));
    } else {
        let file_name = doc
            .file
            .file_name()
            .unwrap()
            .to_str()
            .ok_or_else(|| eyre!("File name contains invalid characters"))?;
        let title = pdf_doc_encode(file_name)?;
        hnd.info.title = Some(PdfString::new(title));
    }
    let creator = pdf_doc_encode("SIGNUM © 1986-93 F. Schmerbeck")?;
    hnd.info.creator = Some(PdfString::new(creator));
    let producer = pdf_doc_encode("Signum! Document Toolbox")?;
    hnd.info.producer = Some(PdfString::new(producer));

    let now = Local::now();
    hnd.info.creation_date = Some(now);
    hnd.info.mod_date = Some(now);

    let use_matrix = doc.use_matrix();
    let pd = doc
        .print_driver
        .ok_or_else(|| eyre!("No printer type selected"))?;
    const FONTS: [&str; 8] = ["C0", "C1", "C2", "C3", "C4", "C5", "C6", "C7"];

    let mut widths: [Vec<u32>; 8] = [
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ];

    let mut first_chars: [u8; 8] = [0; 8];

    let mut fonts = BTreeMap::new();
    for (cset, use_table) in use_matrix.csets.iter().enumerate() {
        match pd {
            FontKind::Printer(pk) => {
                if let Some(pfont) = &doc.chset(&pk, cset) {
                    let name = &doc.chsets[cset]; // FIXME: FontDescriptor
                    let key = FONTS[cset];
                    let efont = doc.chsets_e24[cset].as_deref();
                    if let Some(font) = type3_font(efont, pfont, pk, use_table, Some(name)) {
                        let index = hnd.res.fonts.len();
                        widths[cset] = font.widths.clone();
                        first_chars[cset] = font.first_char;
                        hnd.res.fonts.push(Font::Type3(font));
                        fonts.insert(key.to_owned(), Resource::Global { index });
                    }
                }
            }
            FontKind::Editor => {
                println!("FIXME: Printing with editor fonts is not yet supported");
            }
        }
    }
    hnd.res.font_dicts.push(fonts);

    let media_box = Rectangle::a4_media_box();
    let fscale = (pd.scale() * 500.0) as isize;
    let left = meta.xoffset.unwrap_or(0) as f32;
    let top = 842.0 - meta.yoffset.unwrap_or(0) as f32;
    //100.0 - (pd.scale_x(page_info.margin.left) as f32 * 0.2);

    for (_index, page) in doc.tebu.iter().enumerate() {
        let _page_info = doc.pages[page.index as usize].as_ref().unwrap();

        let mut resources = Resources::default();
        resources.fonts = Resource::Global { index: 0 };

        let mut contents = Contents::new(left, top);

        for (skip, line) in &page.content {
            contents.next_line(0.0, pd.scale_y(1 + skip) as f32 * pd.scale());

            let mut prev_width = 0;
            for te in &line.data {
                let x = te.offset;
                contents.cset(te.cset);

                let diff = pd.scale_x(x) as isize - prev_width;
                if diff != 0 {
                    let xoff = -diff * fscale;
                    contents.xoff(xoff);
                }
                contents.byte(te.cval);

                let csu = te.cset as usize;
                let fc = first_chars[csu];
                let wi = (te.cval - fc) as usize;
                prev_width = widths[csu][wi] as isize;
            }

            contents.flush();
        }

        let page = Page {
            media_box,
            resources,
            contents: contents.into_inner(),
        };
        hnd.pages.push(page);
    }

    Ok(hnd)
}

pub fn output_pdf(doc: &Document) -> eyre::Result<()> {
    let hnd = process_doc(doc)?;

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
            buf.set_extension("pdf");
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
