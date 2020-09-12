use crate::{
    font::printer::FontKind,
    font::{antikro, editor::OwnedESet, printer::OwnedPSet},
    print::Page,
    sdoc::{
        self, parse_cset, parse_hcim, parse_image, parse_pbuf, parse_sdoc0001_container,
        parse_sysp, parse_tebu_header, Flags, Line, Style, Te,
    },
    util::Buf,
    Options,
};
use anyhow::anyhow;
use image::ImageFormat;
use nom::multi::count;
use prettytable::{cell, format, row, Cell, Row, Table};
use sdoc::{parse_page_text, ImageSite, PageText};
use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
    str::FromStr,
};
use thiserror::Error;

struct Pos {
    x: u16,
    y: u16,
}

impl Pos {
    fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

fn print_char_cmds(data: &[Te], x: &mut u16, y: u16) {
    for te in data {
        *x += te.offset;
        println!("({}, {}, {},  {}),", *x, y, te.cval, te.cset);
    }
}

fn print_line_cmds(line: &Line, skip: u16, pos: &mut Pos) {
    pos.x = 0;
    pos.y += (skip + 1) * 2;

    print_char_cmds(&line.data, &mut pos.x, pos.y);
}

#[derive(Copy, Clone)]
pub enum PrintDriver {
    Editor,
    Printer24,
    Laser30,
}

impl PrintDriver {
    fn scale_y(&self, units: u16) -> u32 {
        match self {
            Self::Editor => u32::from(units) * 2,
            Self::Printer24 => u32::from(units) * 6,
            Self::Laser30 => u32::from(units) * 5,
        }
    }

    fn scale_x(&self, units: u16) -> u32 {
        match self {
            Self::Editor => u32::from(units),
            Self::Printer24 => u32::from(units) * 18 / 5,
            Self::Laser30 => u32::from(units) * 7 / 2,
        }
    }
}

#[derive(Debug, Error)]
#[error("Unknown print driver!")]
pub struct UnknownPrintDriver {}

impl FromStr for PrintDriver {
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "E24" => Ok(Self::Editor),
            "P24" => Ok(Self::Printer24),
            "L30" => Ok(Self::Laser30),
            _ => Err(UnknownPrintDriver {}),
        }
    }

    type Err = UnknownPrintDriver;
}

pub struct Document<'a> {
    // Configuration
    print_driver: Option<PrintDriver>,
    opt: &'a Options,
    file: &'a Path,
    // cset
    chsets_e24: [Option<OwnedESet>; 8],
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
    pub fn new(opt: &'a Options, file: &'a Path) -> Self {
        Document {
            opt,
            file,
            chsets_e24: [None, None, None, None, None, None, None, None],
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

    fn print_tebu_data(&self, data: &[Te]) {
        let mut last_char_width: u8 = 0;
        let mut style = Style::default();

        for (_index, k) in data.iter().copied().enumerate() {
            let chr = antikro::decode(k.cval);
            if chr == '\0' {
                println!("<NUL:{}>", k.offset);
                continue;
            }

            if !k.style.bold && style.bold {
                style.bold = false;
                print!("</b>");
            }
            if !k.style.italic && style.italic {
                style.italic = false;
                print!("</i>");
            }
            if !k.style.sth2 && style.sth2 {
                style.sth2 = false;
                print!("</sth2>");
            }
            if !k.style.sth1 && style.sth1 {
                style.sth1 = false;
                print!("</sth1>");
            }
            if !k.style.small && style.small {
                style.small = false;
                print!("</small>");
            }

            let lcw = last_char_width.into();
            if k.offset >= lcw {
                let mut space = k.offset - lcw;

                while space >= 7 {
                    print!(" ");
                    space -= 7;
                }
            }

            if k.style.footnote {
                print!("<footnote>");
            }
            if k.style.small && !style.small {
                style.small = true;
                print!("<small>");
            }
            if k.style.sth1 && !style.sth1 {
                style.sth1 = true;
                print!("<sth1>");
            }
            if k.style.sth2 && !style.sth2 {
                style.sth2 = true;
                print!("<sth2>");
            }
            if k.style.italic && !style.italic {
                style.italic = true;
                print!("<i>");
            }
            if k.style.bold && !style.bold {
                style.bold = true;
                print!("<b>");
            }

            let width = if let Some(eset) = &self.chsets_e24[k.cset as usize] {
                eset.chars[k.cval as usize].width
            } else {
                // default for fonts that are missing
                antikro::WIDTH[k.cval as usize]
            };
            last_char_width = if chr == '\n' { 0 } else { width };
            if (0xE000..=0xE080).contains(&(chr as u32)) {
                print!("<C{}>", (chr as u32) - 0xE000);
            } else if (0x1FBF0..=0x1FBF9).contains(&(chr as u32)) {
                print!("[{}]", chr as u32 - 0x1FBF0);
            } else {
                if k.style.underlined {
                    print!("\u{0332}");
                }
                print!("{}", chr);
            }
        }
        if style.bold {
            print!("</b>");
        }
        if style.italic {
            print!("</i>");
        }
        if style.sth2 {
            print!("</sth2>");
        }
        if style.sth1 {
            print!("</sth1>");
        }
        if style.small {
            print!("</small>");
        }
    }

    fn print_line(&self, line: &Line, skip: u16) {
        if line.flags.contains(Flags::FLAG) && !self.opt.plain {
            println!("<F: {}>", line.extra);
        }

        if line.flags.contains(Flags::PARA) && !self.opt.plain {
            print!("<p>");
        }

        self.print_tebu_data(&line.data);

        if line.flags.contains(Flags::ALIG) && !self.opt.plain {
            print!("<A>");
        }

        if line.flags.contains(Flags::LINE) && !self.opt.plain {
            print!("<br>");
        }

        if self.opt.plain {
            println!();
        } else {
            println!("{{{}}}", skip);
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
        kind: FontKind,
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
            (Ok(pset), FontKind::Needle24) => {
                self.chsets_p24[index] = Some(pset);
                println!("Loaded font file '{}'", printer_cset_file.display());
                true
            }
            (Ok(pset), FontKind::Laser30) => {
                self.chsets_l30[index] = Some(pset);
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

    fn process_cset(&mut self, part: Buf) -> anyhow::Result<()> {
        let (_, charsets) = parse_cset(part.0).unwrap();
        println!("'cset': {:?}", charsets);

        let folder = self.file.parent().unwrap();
        let default_cset_folder = folder.join("CHSETS");

        let mut all_eset = true;
        let mut all_pset = true;
        let mut all_lset = true;
        for (index, name) in charsets.into_iter().enumerate() {
            if name.is_empty() {
                continue;
            }
            let cset_folder = default_cset_folder.as_path();
            let name_ref = name.as_ref();
            all_eset &= self.load_cset_editor(index, cset_folder, name_ref);
            all_pset &= self.load_cset_printer(index, cset_folder, name_ref, FontKind::Needle24);
            all_lset &= self.load_cset_printer(index, cset_folder, name_ref, FontKind::Laser30);
        }
        // Print info on which sets are available
        if all_eset {
            println!("Editor fonts available for all character sets");
        }
        if all_pset {
            println!("Printer fonts (24-needle) available for all character sets");
        }
        if all_lset {
            //println!("Printer fonts (laser/30) available for all character sets");
        }

        // If none was set, choose one strategy
        if let Some(pd) = self.print_driver {
            match pd {
                PrintDriver::Editor => {
                    if !all_eset {
                        println!("WARNING: Explicitly chosen editor print-driver but not all fonts are available");
                    }
                }
                PrintDriver::Printer24 => {
                    if !all_pset {
                        println!("WARNING: Explicitly chosen 24-needle print-driver but not all fonts are available");
                    }
                }
                PrintDriver::Laser30 => {
                    if !all_lset {
                        println!("WARNING: Explicitly chosen laser/30 print-driver but not all fonts are available");
                    }
                }
            }
        } else if all_lset {
            self.print_driver = Some(PrintDriver::Laser30);
        } else if all_pset {
            self.print_driver = Some(PrintDriver::Printer24);
        } else if all_eset {
            self.print_driver = Some(PrintDriver::Editor);
        } else {
            println!("No print-driver has all fonts available.");
        }
        Ok(())
    }

    fn process_pbuf(&mut self, part: Buf) -> anyhow::Result<()> {
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

    fn draw_chars(&self, data: &[Te], page: &mut Page, x: &mut u16, y: u16) -> anyhow::Result<()> {
        for te in data {
            *x += te.offset;
            match self.print_driver {
                Some(PrintDriver::Editor) => {
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
                Some(PrintDriver::Printer24) => {
                    if let Some(eset) = &self.chsets_p24[te.cset as usize] {
                        let ch = &eset.chars[te.cval as usize];
                        let x = (*x as u32) * 18 / 5;
                        let y = (y as u32) * 6;
                        match page.draw_printer_char(x, y, ch) {
                            Ok(()) => {}
                            Err(()) => {
                                eprintln!("Char out of bounds {:?}", te);
                            }
                        }
                    }
                }
                Some(PrintDriver::Laser30) => {
                    if let Some(eset) = &self.chsets_l30[te.cset as usize] {
                        let ch = &eset.chars[te.cval as usize];
                        // 405 DPI
                        let x = (*x as u32) * 7 / 2;
                        let y = (y as u32) * 5;
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
    ) -> anyhow::Result<()> {
        pos.y += skip + 1;

        if line.flags.contains(Flags::FLAG) {
            println!("<F: {}>", line.extra);
        }

        if line.flags.contains(Flags::ALIG) {}

        self.draw_chars(&line.data, page, &mut pos.x, pos.y)?;

        Ok(())
    }

    fn process_tebu(&mut self, part: Buf) -> anyhow::Result<()> {
        let (rest, tebu_header) = parse_tebu_header(part.0).unwrap();
        println!("'tebu': {:?}", tebu_header);

        let (rest, tebu) = match count(parse_page_text, self.page_count)(rest) {
            Ok(r) => r,
            Err(e) => {
                return Err(anyhow!("Failed to process pages: {}", e));
            }
        };
        self.tebu = tebu;
        println!("Loaded all pages!");

        if !rest.is_empty() {
            println!("{:#?}", Buf(rest));
        }
        Ok(())
    }

    fn process_hcim(&mut self, part: Buf) -> anyhow::Result<()> {
        let (rest, hcim) = parse_hcim(part.0).unwrap();
        println!("'hcim':");
        println!("  {:?}", hcim.header);

        let out_img = self.opt.imout.as_ref();
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

    fn output_print(&self, out_path: &Path) -> anyhow::Result<()> {
        for page_text in &self.tebu {
            let index = page_text.index as usize;
            let pbuf_entry = self.pages[index].as_ref().unwrap();

            println!("{}", page_text.skip);

            if let Some(pages) = &self.opt.page {
                if !pages.contains(&(pbuf_entry.log_pnr as usize)) {
                    continue;
                }
            }

            let (mut page, mut pos) = if let Some(print_driver) = self.print_driver {
                let width_units: u16 = pbuf_entry.margin.left + pbuf_entry.margin.right + 20;
                let height_units: u16 =
                    pbuf_entry.margin.top + pbuf_entry.lines + pbuf_entry.margin.bottom;

                let width = print_driver.scale_x(width_units);
                let height = print_driver.scale_y(height_units);

                let page = Page::new(width, height);
                let pos = Pos::new(10, 0 /*page_text.skip & 0x00FF*/);
                (page, pos)
            } else {
                println!(
                    "Print Driver not set, skipping page #{}",
                    pbuf_entry.log_pnr
                );
                continue;
            };

            for (skip, line) in &page_text.content {
                pos.x = 10;
                self.draw_line(line, *skip, &mut page, &mut pos)?;
            }

            for site in self.sites.iter().filter(|x| x.page == pbuf_entry.phys_pnr) {
                println!(
                    "{}x{}+{},{} of {} at {},{}",
                    site.sel.w,
                    site.sel.h,
                    site.sel.x,
                    site.sel.y,
                    site.img,
                    site.pos_x,
                    site.pos_y
                );

                if let Some(pd) = self.print_driver {
                    let px = pd.scale_x(10 + site.pos_x);
                    let w = pd.scale_x(site._3);
                    let py = pd.scale_y(10 + site.pos_y - site._5 / 2);
                    let h = pd.scale_y(site._4 / 2);
                    let image = &self.images[site.img as usize];
                    page.draw_image(px, py, w, h, image, site.sel);
                }
            }

            let image = page.to_image();
            let file_name = format!("page-{}.png", pbuf_entry.log_pnr);
            println!("Saving {}", file_name);
            let page_path = out_path.join(&file_name);
            image.save_with_format(&page_path, ImageFormat::Png)?;
        }
        Ok(())
    }

    fn output(&self) -> anyhow::Result<()> {
        if let Some(out_path) = &self.opt.out {
            self.output_print(out_path)?;
        } else if self.opt.pdraw {
            for page_text in &self.tebu {
                let mut pos = Pos::new(0, 0);
                for (skip, line) in &page_text.content {
                    print_line_cmds(&line, *skip, &mut pos);
                }
            }
        } else {
            for page_text in &self.tebu {
                let index = page_text.index as usize;
                let pbuf_entry = self.pages[index].as_ref().unwrap();
                println!(
                    "{:04X} ----------------- [PAGE {} ({})] -------------------",
                    page_text.skip, pbuf_entry.log_pnr, pbuf_entry.phys_pnr
                );
                for (skip, line) in &page_text.content {
                    self.print_line(line, *skip);
                }
                println!(
                    "{:04X} -------------- [END OF PAGE {} ({})] ---------------",
                    page_text.skip, pbuf_entry.log_pnr, pbuf_entry.phys_pnr
                );
            }
        }
        Ok(())
    }
}

pub fn process_sdoc(buffer: &[u8], opt: Options, file: &Path) -> anyhow::Result<()> {
    let (rest, sdoc) = match parse_sdoc0001_container(buffer) {
        Ok(x) => x,
        Err(nom::Err::Failure((rest, kind))) => {
            return Err(anyhow!("Parse failed [{:?}]:\n{:?}", rest, kind));
        }
        Err(nom::Err::Error((rest, kind))) => {
            return Err(anyhow!("Parse errored [{:?}]:\n{:?}", rest, kind));
        }
        Err(nom::Err::Incomplete(a)) => {
            return Err(anyhow!("Parse incomplete, needed {:?}", a));
        }
    };

    let mut document = Document::new(&opt, file);

    if let Some(out_path) = &opt.out {
        std::fs::create_dir_all(out_path)?;
    }

    for (key, part) in sdoc.parts {
        match key {
            "cset" => {
                document.process_cset(part)?;
            }
            "sysp" => {
                let (_, sysp) = parse_sysp(part.0).unwrap();
                println!("'sysp': {:#?}", sysp);
            }
            "pbuf" => {
                document.process_pbuf(part)?;
            }
            "tebu" => {
                document.process_tebu(part)?;
            }
            "hcim" => {
                document.process_hcim(part)?;
            }
            _ => {
                println!("'{}': {}", key, part.0.len());
            }
        }
    }

    // Output the document
    document.output()?;

    if !rest.is_empty() {
        println!("remaining: {:#?}", Buf(rest));
    }
    Ok(())
}
