use std::{borrow::Cow, collections::BTreeMap, fs::File, io::BufWriter, path::Path};

use color_eyre::eyre::{self, eyre};
use pdf_create::{
    chrono::Local,
    common::{OutputIntent, OutputIntentSubtype, PdfString, Rectangle},
    encoding::pdf_doc_encode,
    high::{Font, Handle, Page, Resource, Resources},
};
use sdo_pdf::{font::type3_font, sdoc::Contents};
use signum::chsets::{printer::PrinterKind, FontKind, UseTableVec};

use crate::cli::{font::cache::FontCache, opt::Meta};

use super::Document;

struct FontInfo {
    widths: Vec<u32>,
    first_char: u8,
    index: usize,
}

pub struct Fonts {
    info: Vec<Option<FontInfo>>,
    base: usize,
}

impl Fonts {
    pub fn new(fonts_capacity: usize, base: usize) -> Self {
        Fonts {
            info: Vec::with_capacity(fonts_capacity),
            base,
        }
    }

    pub fn make_fonts<'a>(
        &mut self,
        fc: &'a FontCache,
        use_table_vec: UseTableVec,
        pk: PrinterKind,
    ) -> eyre::Result<Vec<Font<'a>>> {
        let chsets = fc.chsets();
        let mut result = Vec::with_capacity(chsets.len());
        for (index, cs) in chsets.iter().enumerate() {
            let use_table = &use_table_vec.csets[index];

            if let Some(pfont) = cs.printer(pk) {
                // FIXME: FontDescriptor

                let efont = cs.e24();
                if let Some(font) = type3_font(efont, pfont, pk, use_table, Some(cs.name())) {
                    let info = FontInfo {
                        widths: font.widths.clone(),
                        first_char: font.first_char,
                        index: result.len(),
                    };
                    self.info.push(Some(info));
                    result.push(Font::Type3(font));
                    continue;
                }
            }
            self.info.push(None);
        }
        Ok(result)
    }
}

pub fn prepare_meta(hnd: &mut Handle, meta: &Meta) -> eyre::Result<()> {
    // Metadata
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
    }
    let creator = pdf_doc_encode("SIGNUM © 1986-93 F. Schmerbeck")?;
    hnd.info.creator = Some(PdfString::new(creator));
    let producer = pdf_doc_encode("Signum! Document Toolbox")?;
    hnd.info.producer = Some(PdfString::new(producer));

    let now = Local::now();
    hnd.info.creation_date = Some(now);
    hnd.info.mod_date = Some(now);

    // Output intents
    hnd.output_intents.push(OutputIntent {
        subtype: OutputIntentSubtype::GTS_PDFA1,
        output_condition: None,
        output_condition_identifier: PdfString::new("FOO"),
        registry_name: None,
        info: None,
    });

    Ok(())
}

const FONTS: [&str; 8] = ["C0", "C1", "C2", "C3", "C4", "C5", "C6", "C7"];

pub fn prepare_document(
    hnd: &mut Handle,
    doc: &Document,
    meta: &Meta,
    font_info: &Fonts,
) -> eyre::Result<()> {
    let mut fonts = BTreeMap::new();
    let mut infos = [None; 8];
    for (cset, fc_index) in doc.chsets.iter().copied().enumerate() {
        if let Some(fc_index) = fc_index {
            let key = FONTS[cset].to_owned();
            if let Some(info) = &font_info.info[fc_index] {
                let index = font_info.base + info.index;
                let value = Resource::Global { index };
                fonts.insert(key, value);
                infos[cset] = Some(info);
            }
        }
    }

    hnd.res.font_dicts.push(fonts);

    // PDF uses a unit length of 1/72 1/(18*4) of an inch by default
    //
    // Signum uses 1/54 1/(18*3) of an inch vertically and 1/90 1/(18*5) horizontally

    for (_index, page) in doc.tebu.iter().enumerate() {
        let page_info = doc.pages[page.index as usize].as_ref().unwrap();

        let resources = Resources {
            fonts: Resource::Global { index: 0 },
            ..Default::default()
        };

        let a4_width = 592;
        let a4_height = 842;

        let width = page_info.format.width() * 72 / 90;
        let height = page_info.format.length as i32 * 72 / 54;

        assert!(width as i32 <= a4_width, "Please file a bug!");

        let xmargin = (a4_width - width as i32) / 2;
        let ymargin = (a4_height - height as i32) / 2;

        let left = xmargin as f32 + meta.xoffset.unwrap_or(0) as f32;
        let left = left - page_info.format.left as f32 * 8.0 / 10.0;
        let top = ymargin as f32 + meta.yoffset.unwrap_or(0) as f32;
        let top = a4_height as f32 - top - 8.0;
        let media_box = Rectangle::media_box(a4_width, a4_height);

        let mut contents = Contents::new(1.0, -1.0, left, top);

        for (skip, line) in &page.content {
            contents.next_line(0, *skip as u32 + 1);

            const FONTUNITS_PER_SIGNUM_X: i32 = 800;
            let mut prev_width = 0;
            for te in &line.data {
                let x = te.offset as i32;
                contents.cset(te.cset);

                let diff = x * FONTUNITS_PER_SIGNUM_X - prev_width;
                if diff != 0 {
                    contents.xoff(-diff);
                }
                contents.byte(te.cval);

                let csu = te.cset as usize;
                let fi = infos[csu].ok_or_else(|| {
                    let font_name = doc.cset[csu].as_deref().unwrap_or("");
                    eyre!("Missing font #{}: {:?}", csu, font_name)
                })?;
                let fc = fi.first_char;
                let wi = (te.cval - fc) as usize;
                prev_width = fi.widths[wi] as i32;
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

    Ok(())
}

fn doc_meta<'a>(doc: &'a Document) -> eyre::Result<Cow<'a, Meta>> {
    let meta = doc.opt.meta()?;
    if meta.title.is_none() {
        let mut meta = meta.into_owned();
        let file_name = doc.opt.file.file_name().unwrap();
        let title = file_name
            .to_str()
            .ok_or_else(|| eyre!("File name contains invalid characters"))?;
        meta.title = Some(title.to_owned());
        Ok(Cow::Owned(meta))
    } else {
        Ok(meta)
    }
}

pub fn process_doc<'a>(doc: &'a Document, fc: &'a FontCache) -> eyre::Result<Handle<'a>> {
    let mut hnd = Handle::new();

    let meta = doc_meta(doc)?;
    prepare_meta(&mut hnd, &meta)?;

    let use_matrix = doc.use_matrix();
    let mut use_table_vec = UseTableVec::new();
    use_table_vec.append(&doc.chsets, use_matrix);

    let pd = doc
        .print_driver
        .ok_or_else(|| eyre!("No printer type selected"))?;

    let pk = if let FontKind::Printer(pk) = pd {
        pk
    } else {
        return Err(eyre!("Editor fonts are not currently supported"));
    };

    let mut font_info = Fonts {
        info: Vec::with_capacity(8),
        base: hnd.res.fonts.len(),
    };

    for font in font_info.make_fonts(fc, use_table_vec, pk)? {
        hnd.res.fonts.push(font);
    }

    prepare_document(&mut hnd, doc, &meta, &font_info)?;
    Ok(hnd)
}

pub fn output_pdf(doc: &Document, fc: &FontCache) -> eyre::Result<()> {
    let hnd = process_doc(doc, fc)?;
    handle_out(doc.opt.out.as_deref(), &doc.opt.file, hnd)?;
    Ok(())
}

pub fn handle_out(out: Option<&Path>, file: &Path, hnd: Handle) -> eyre::Result<()> {
    if out == Some(Path::new("-")) {
        println!("----------------------------- PDF -----------------------------");
        let stdout = std::io::stdout();
        let mut stdolock = stdout.lock();
        hnd.write(&mut stdolock)?;
        println!("---------------------------------------------------------------");
        Ok(())
    } else {
        let out = out.unwrap_or_else(|| file.parent().unwrap());
        let file = file.file_stem().unwrap();
        let out = {
            let mut buf = out.join(file);
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
