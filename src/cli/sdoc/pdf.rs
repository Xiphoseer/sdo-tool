use std::{collections::BTreeMap, fmt::Write, fs::File, io::BufWriter, path::Path};

use color_eyre::eyre::{self, eyre};
use pdf::primitive::PdfString;
use pdf_create::{
    common::Rectangle,
    high::{Font, Handle, Page, Resource, Resources},
};
use sdo::font::FontKind;
use sdo_pdf::font::type3_font;

use super::Document;

struct Contents {
    buf: Vec<u8>,
    inner: String,
    cset: u8,
    open: bool,
    needs_space: bool,
    is_ascii: bool,
    line_started: bool,
    line_y: f32,
    line_x: f32,
}

impl Contents {
    fn new(left: f32, top: f32) -> Self {
        Contents {
            line_started: false,
            line_y: 0.0,
            line_x: 0.0,
            buf: vec![],
            open: false,
            needs_space: false,
            is_ascii: true,
            cset: 0xff,
            inner: format!("0 g\nBT\n1 0 0 -1 {} {} Tm\n", left, top),
        }
    }

    fn next_line(&mut self, x: f32, y: f32) {
        self.line_x += x;
        self.line_y += y;
        self.line_started = false;
    }

    fn start_line(&mut self) {
        if !self.line_started {
            self.line_started = true;
            writeln!(self.inner, "{} {} Td", self.line_x, self.line_y).unwrap();
            self.line_y = 0.0;
        }
    }

    fn cset(&mut self, cset: u8) {
        if self.cset != cset {
            self.cset = cset;
            self.flush();
            writeln!(self.inner, "/C{} 1 Tf", cset).unwrap();
        }
    }

    fn xoff(&mut self, xoff: isize) {
        self.open();
        self.buf_flush();
        if self.needs_space {
            write!(self.inner, " ").unwrap();
        }
        write!(self.inner, "{}", xoff).unwrap();
        self.needs_space = true;
    }

    fn byte(&mut self, byte: u8) {
        self.open();
        self.buf.push(byte);
        self.is_ascii = self.is_ascii && (byte > 31) && (byte < 127);
    }

    fn buf_flush(&mut self) {
        if self.buf.is_empty() {
            return;
        }
        if self.is_ascii {
            self.inner.push('(');
            for b in self.buf.drain(..) {
                if matches!(b, 0x28 | 0x29 | 0x5c) {
                    self.inner.push('\\');
                }
                self.inner.push(b as char);
            }
            self.inner.push(')');
        } else {
            write!(self.inner, "<").unwrap();
            for byte in self.buf.drain(..) {
                write!(self.inner, "{:02X}", byte).unwrap();
            }
            write!(self.inner, ">").unwrap();
        }
        self.is_ascii = true;
        self.needs_space = false;
    }

    fn open(&mut self) {
        if !self.open {
            self.start_line();
            write!(self.inner, "[").unwrap();
            self.open = true;
            self.needs_space = false;
        }
    }

    fn flush(&mut self) {
        if self.open {
            self.open = false;
            self.buf_flush();
            writeln!(self.inner, "] TJ").unwrap();
        }
    }

    fn into_inner(mut self) -> String {
        self.inner.push_str("ET\n");
        self.inner
    }
}

pub fn process_doc<'a>(doc: &'a Document) -> eyre::Result<Handle<'a>> {
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

    // FIXME START

    let media_box = Rectangle::a4_media_box();
    let fscale = (pd.scale() * 1000.0) as isize;

    for (_index, page) in doc.tebu.iter().enumerate() {
        let _page_info = doc.pages[page.index as usize].as_ref().unwrap();

        let mut resources = Resources::default();
        resources.fonts = Resource::Global { index: 0 };

        //100.0 - (pd.scale_x(page_info.margin.left) as f32 * 0.2);
        let left = doc.opt.xoffset.unwrap_or(0) as f32;
        let top = 842.0 - doc.opt.yoffset.unwrap_or(0) as f32;
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

        // FIXME END
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
