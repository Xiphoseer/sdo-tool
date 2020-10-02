use crate::cli::opt::{Format, Options};
use color_eyre::eyre::{self, eyre};
use image::ImageFormat;
use nom::Finish;
use prettytable::{cell, format, row, Cell, Row, Table};
use sdo::{
    font::{
        editor::OwnedESet,
        printer::{OwnedPSet, PSet, PrinterKind},
        FontKind,
    },
    print::Page,
    sdoc::{
        self, parse_cset, parse_hcim, parse_image, parse_pbuf, parse_sdoc0001_container,
        parse_sysp, parse_tebu_header, Flags, Line, Te,
    },
    util::Buf,
};
use sdo::{
    nom::{self, multi::count},
    sdoc::{parse_page_text, ImageSite, PageText},
};
use std::{
    borrow::Cow,
    fs::DirEntry,
    path::{Path, PathBuf},
};

mod console;
mod imgseq;
mod pdraw;
mod ps;
mod ps_proc;

struct Pos {
    x: u16,
    y: u16,
}

impl Pos {
    fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

pub struct Document<'a> {
    // Configuration
    print_driver: Option<FontKind>,
    opt: &'a Options,
    file: &'a Path,
    // cset
    chsets: Vec<Cow<'a, str>>,
    chsets_e24: [Option<OwnedESet>; 8],
    chsets_p9: [Option<OwnedPSet>; 8],
    chsets_p24: [Option<OwnedPSet>; 8],
    chsets_l30: [Option<OwnedPSet>; 8],
    // pbuf
    pages: Vec<Option<sdoc::Page>>,
    page_count: usize,
    // tebu
    tebu: Vec<PageText>,
    // hcim
    images: Vec<Page>,
    sites: Vec<ImageSite>,
}

impl<'a> Document<'a> {
    fn chset<'b>(&'b self, pd: &PrinterKind, cset: u8) -> Option<&'b PSet<'a>> {
        match pd {
            PrinterKind::Needle9 => self.chsets_p9[cset as usize].as_deref(),
            PrinterKind::Needle24 => self.chsets_p24[cset as usize].as_deref(),
            PrinterKind::Laser30 => self.chsets_l30[cset as usize].as_deref(),
        }
    }

    pub fn use_matrix(&self) -> [[usize; 128]; 8] {
        let mut use_matrix = [[0; 128]; 8];

        for page in &self.tebu {
            for (_, line) in &page.content {
                for tw in &line.data {
                    use_matrix[tw.cset as usize][tw.cval as usize] += 1;
                }
            }
        }

        use_matrix
    }

    pub fn new(opt: &'a Options, file: &'a Path) -> Self {
        Document {
            opt,
            file,
            chsets: vec![],
            chsets_e24: [None, None, None, None, None, None, None, None],
            chsets_p9: [None, None, None, None, None, None, None, None],
            chsets_p24: [None, None, None, None, None, None, None, None],
            chsets_l30: [None, None, None, None, None, None, None, None],
            pages: vec![],
            page_count: 0,
            print_driver: opt.print_driver,
            tebu: vec![],
            images: vec![],
            sites: vec![],
        }
    }

    fn find_font_file(cset_folder: &Path, name: &str, extension: &str) -> Option<PathBuf> {
        let cset_file = cset_folder.join(name);
        let editor_cset_file = cset_file.with_extension(extension);

        if editor_cset_file.exists() && editor_cset_file.is_file() {
            return Some(editor_cset_file);
        }

        let mut dir_iter = match std::fs::read_dir(cset_folder) {
            Ok(i) => i,
            Err(e) => {
                println!("Could not find CHSET folder: {}", e);
                return None;
            }
        };

        let file = dir_iter.find_map(|entry| {
            entry
                .ok()
                .as_ref()
                .map(DirEntry::path)
                .filter(|p| p.is_dir())
                .and_then(|cset_folder| Self::find_font_file(&cset_folder, name, extension))
        });

        if let Some(file) = file {
            Some(file)
        } else {
            None
        }
    }

    fn load_cset_editor(&mut self, index: usize, cset_folder: &Path, name: &str) -> bool {
        let editor_cset_file = match Self::find_font_file(cset_folder, name, "E24") {
            Some(f) => f,
            None => {
                println!("Editor font for `{}` not found!", name);
                return false;
            }
        };

        match OwnedESet::load(&editor_cset_file) {
            Ok(eset) => {
                self.chsets_e24[index] = Some(eset);
                println!("Loaded font file '{}'", editor_cset_file.display());
                true
            }
            Err(e) => {
                println!("Failed to parse font file {}", editor_cset_file.display());
                println!("Are you sure this is a valid Signum! editor font?");
                println!("Error: {}", e);
                false
            }
        }
    }

    fn load_cset_printer(
        &mut self,
        index: usize,
        cset_folder: &Path,
        name: &str,
        kind: PrinterKind,
    ) -> bool {
        let extension = kind.extension();
        let printer_cset_file = match Self::find_font_file(cset_folder, name, extension) {
            Some(f) => f,
            None => {
                println!("Printer font file '{}.{}' not found", name, extension);
                return false;
            }
        };

        match (OwnedPSet::load(&printer_cset_file, kind), kind) {
            (Ok(pset), PrinterKind::Needle24) => {
                self.chsets_p24[index] = Some(pset);
                println!("Loaded font file '{}'", printer_cset_file.display());
                true
            }
            (Ok(pset), PrinterKind::Laser30) => {
                self.chsets_l30[index] = Some(pset);
                println!("Loaded font file '{}'", printer_cset_file.display());
                true
            }
            (Ok(pset), PrinterKind::Needle9) => {
                self.chsets_p9[index] = Some(pset);
                println!("Loaded font file '{}'", printer_cset_file.display());
                true
            }
            (Err(e), _) => {
                println!("Failed to parse font file {}", printer_cset_file.display());
                println!("Are you sure this is a valid Signum! editor font?");
                println!("Error: {}", e);
                false
            }
        }
    }

    fn process_cset(&mut self, part: Buf<'a>) -> eyre::Result<()> {
        let (_, charsets) = parse_cset(part.0).unwrap();
        println!("'cset': {:?}", charsets);

        let folder = self.file.parent().unwrap();
        let default_cset_folder = folder.join("CHSETS");

        let mut all_eset = true;
        let mut all_pset = true;
        let mut all_lset = true;
        let mut all_p9 = true;
        for (index, name) in charsets.iter().enumerate() {
            if name.is_empty() {
                continue;
            }
            let cset_folder = default_cset_folder.as_path();
            let name_ref = name.as_ref();
            all_eset &= self.load_cset_editor(index, cset_folder, name_ref);
            all_pset &= self.load_cset_printer(index, cset_folder, name_ref, PrinterKind::Needle24);
            all_lset &= self.load_cset_printer(index, cset_folder, name_ref, PrinterKind::Laser30);
            all_p9 &= self.load_cset_printer(index, cset_folder, name_ref, PrinterKind::Needle9);
        }
        all_p9 = false;
        // Print info on which sets are available
        if all_eset {
            println!("Editor fonts available for all character sets");
        }
        if all_pset {
            println!("Printer fonts (24-needle) available for all character sets");
        }
        if all_lset {
            println!("Printer fonts (laser/30) available for all character sets");
        }
        if all_p9 {
            println!("Printer fonts (9-needle) available for all character sets");
        }

        // If none was set, choose one strategy
        if let Some(pd) = self.print_driver {
            match pd {
                FontKind::Editor => {
                    if !all_eset {
                        println!("WARNING: Explicitly chosen editor print-driver but not all fonts are available");
                    }
                }
                FontKind::Printer(PrinterKind::Needle24) => {
                    if !all_pset {
                        println!("WARNING: Explicitly chosen 24-needle print-driver but not all fonts are available");
                    }
                }
                FontKind::Printer(PrinterKind::Needle9) => {
                    if !all_p9 {
                        println!("WARNING: Explicitly chosen 9-needle print-driver but not all fonts are available");
                    }
                }
                FontKind::Printer(PrinterKind::Laser30) => {
                    if !all_lset {
                        println!("WARNING: Explicitly chosen laser/30 print-driver but not all fonts are available");
                    }
                }
            }
        } else if all_lset {
            self.print_driver = Some(FontKind::Printer(PrinterKind::Laser30));
        } else if all_pset {
            self.print_driver = Some(FontKind::Printer(PrinterKind::Needle24));
        } else if all_p9 {
            self.print_driver = Some(FontKind::Printer(PrinterKind::Needle9));
        } else if all_eset {
            self.print_driver = Some(FontKind::Editor);
        } else {
            println!("No print-driver has all fonts available.");
        }
        self.chsets = charsets;
        Ok(())
    }

    fn process_sysp(&mut self, part: Buf) -> eyre::Result<()> {
        let (_, sysp) = parse_sysp(part.0)
            .finish()
            .map_err(|e| eyre!("Failed to parse `sysp`: {:?}", e))?;
        println!("'sysp': {:#?}", sysp);
        Ok(())
    }

    fn process_pbuf(&mut self, part: Buf) -> eyre::Result<()> {
        let (rest, pbuf) = parse_pbuf(part.0).unwrap();

        println!(
            "Page Buffer ('pbuf')\n  page_count: {}\n  kl: {}\n  first_page_nr: {}",
            pbuf.page_count, pbuf.kl, pbuf.first_page_nr
        );

        if !rest.is_empty() {
            println!("  rest: {:+?}", Buf(rest));
        }

        // Create the table
        let mut page_table = Table::new();
        page_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        // Add a row per time
        page_table.set_titles(row![
            "idx", "#phys", "#log", "lines", "m-l", "m-r", "m-t", "m-b", "numbpos", "kapitel",
            "intern", "rest",
        ]);

        for (index, pbuf_entry) in pbuf.pages.iter().enumerate() {
            if let Some((page, buf)) = pbuf_entry {
                page_table.add_row(row![
                    index,
                    page.phys_pnr,
                    page.log_pnr,
                    page.lines,
                    page.margin.left,
                    page.margin.right,
                    page.margin.top,
                    page.margin.bottom,
                    page.numbpos,
                    page.kapitel,
                    page.intern,
                    buf,
                ]);
            } else {
                page_table.add_row(row![
                    index, "---", "---", "---", "---", "---", "---", "---", "---", "---", "---",
                    "---"
                ]);
            }
        }

        // Print the table to stdout
        page_table.printstd();

        self.pages = pbuf.pages.into_iter().map(|f| f.map(|(p, _b)| p)).collect();
        self.page_count = pbuf.page_count as usize;

        Ok(())
    }

    fn draw_chars(&self, data: &[Te], page: &mut Page, x: &mut u16, y: u16) -> eyre::Result<()> {
        for te in data {
            *x += te.offset;
            match self.print_driver {
                Some(FontKind::Editor) => {
                    if let Some(eset) = &self.chsets_e24[te.cset as usize] {
                        let ch = &eset.chars[te.cval as usize];
                        let x = *x; // No skew compensation (18/15)
                        let y = y * 2;
                        match page.draw_echar(x, y, ch) {
                            Ok(()) => {}
                            Err(()) => {
                                eprintln!("Char out of bounds {:?}", te);
                            }
                        }
                    }
                }
                Some(FontKind::Printer(pk)) => {
                    if let Some(eset) = self.chset(&pk, te.cset) {
                        let ch = &eset.chars[te.cval as usize];
                        let fk = FontKind::Printer(pk); // FIXME: pattern after @-binding
                        let x = fk.scale_x(*x);
                        let y = fk.scale_y(y);
                        match page.draw_printer_char(x, y, ch) {
                            Ok(()) => {}
                            Err(()) => {
                                eprintln!("Char out of bounds {:?}", te);
                            }
                        }
                    }
                }
                None => {
                    continue;
                }
            }
        }
        Ok(())
    }

    fn draw_line(
        &self,
        line: &Line,
        skip: u16,
        page: &mut Page,
        pos: &mut Pos,
    ) -> eyre::Result<()> {
        pos.y += skip + 1;

        if line.flags.contains(Flags::FLAG) {
            println!("<F: {}>", line.extra);
        }

        if line.flags.contains(Flags::ALIG) {}

        self.draw_chars(&line.data, page, &mut pos.x, pos.y)?;

        Ok(())
    }

    fn process_tebu(&mut self, part: Buf) -> eyre::Result<()> {
        let (rest, tebu_header) = parse_tebu_header(part.0).unwrap();
        println!("'tebu': {:?}", tebu_header);

        let (rest, tebu) = match count(parse_page_text, self.page_count)(rest) {
            Ok(r) => r,
            Err(e) => {
                return Err(eyre!("Failed to process pages: {}", e));
            }
        };
        self.tebu = tebu;
        println!("Loaded all pages!");

        if !rest.is_empty() {
            println!("{:#?}", Buf(rest));
        }
        Ok(())
    }

    fn process_hcim(&mut self, part: Buf) -> eyre::Result<()> {
        let (rest, hcim) = parse_hcim(part.0).unwrap();
        println!("'hcim':");
        println!("  {:?}", hcim.header);

        let out_img = self.opt.with_images.as_ref();
        if let Some(out_img) = out_img {
            std::fs::create_dir_all(out_img)?;
        }

        let mut image_table = Table::new();
        image_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        // Add a row per time
        image_table.set_titles(row![
            "page", "pos_x", "pos_y", "[3]", "[4]", "[5]", "sel_x", "sel_y", "sel_w", "sel_h",
            "[A]", "[B]", "[C]", "img", "[E]", "[F]",
        ]);

        for isite in &hcim.sites {
            image_table.add_row(Row::new(vec![
                Cell::new(&format!("{}", isite.page)),
                Cell::new(&format!("{}", isite.pos_x)),
                Cell::new(&format!("{}", isite.pos_y)),
                Cell::new(&format!("{}", isite._3)),
                Cell::new(&format!("{}", isite._4)),
                Cell::new(&format!("{}", isite._5)),
                Cell::new(&format!("{}", isite.sel.x)),
                Cell::new(&format!("{}", isite.sel.y)),
                Cell::new(&format!("{}", isite.sel.w)),
                Cell::new(&format!("{}", isite.sel.h)),
                Cell::new(&format!("{}", isite._A)),
                Cell::new(&format!("{}", isite._B)),
                Cell::new(&format!("{}", isite._C)),
                Cell::new(&format!("{}", isite.img)),
                Cell::new(&format!("{}", isite._E)),
                Cell::new(&format!("{:?}", isite._F)),
            ]));
        }

        image_table.printstd();

        let mut images = Vec::with_capacity(hcim.header.img_count as usize);

        for (index, img) in hcim.images.iter().enumerate() {
            println!("image[{}]:", index);
            match parse_image(img.0) {
                Ok((_imgrest, im)) => {
                    println!("IMAGE: {:?}", im.key);
                    println!("{:#?}", im.bytes);
                    let page = Page::from_screen(im.image);
                    if let Some(out_img) = out_img {
                        let name = format!("{:02}-{}.png", index, im.key);
                        let path = out_img.join(name);
                        let img = page.to_image();
                        img.save_with_format(&path, ImageFormat::Png)?;
                    }
                    images.push(page);
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }

        self.images = images;
        self.sites = hcim.sites;

        if !rest.is_empty() {
            println!("{:#?}", Buf(rest));
        }

        Ok(())
    }

    fn output(&self) -> eyre::Result<()> {
        match self.opt.format {
            Format::Html | Format::Plain => console::output_console(self),
            Format::PostScript => ps::output_postscript(self),
            Format::PDraw => pdraw::output_pdraw(self),
            Format::Png => imgseq::output_print(self),
            Format::DVIPSBitmapFont | Format::CCITTT6 => {
                panic!("Document can't be formatted as a font")
            }
        }
    }
}

pub fn process_sdoc(buffer: &[u8], opt: Options, file: &Path) -> eyre::Result<()> {
    let (rest, sdoc) = parse_sdoc0001_container(buffer)
        .finish()
        .map_err(|e| eyre!("Parse failed [{:?}]:\n{:?}", e.input, e.code))?;

    let mut document = Document::new(&opt, file);

    if opt.out != Path::new("-") {
        std::fs::create_dir_all(&opt.out)?;
    }

    for (key, part) in sdoc.parts {
        match key {
            "cset" => document.process_cset(part),
            "sysp" => document.process_sysp(part),
            "pbuf" => document.process_pbuf(part),
            "tebu" => document.process_tebu(part),
            "hcim" => document.process_hcim(part),
            _ => {
                println!("'{}': {}", key, part.0.len());
                Ok(())
            }
        }?;
    }

    // Output the document
    document.output()?;

    if !rest.is_empty() {
        println!("remaining: {:#?}", Buf(rest));
    }
    Ok(())
}
