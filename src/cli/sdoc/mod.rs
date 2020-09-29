use crate::cli::opt::{Format, Options};
use color_eyre::eyre::{self, eyre};
use image::ImageFormat;
use nom::Finish;
use prettytable::{cell, format, row, Cell, Row, Table};
use ps::prog_dict;
use sdo::{
    font::printer::FontKind,
    font::printer::PSet,
    font::{antikro, editor::OwnedESet, printer::OwnedPSet, printer::PrintDriver},
    print::Page,
    ps::PSWriter,
    sdoc::{
        self, parse_cset, parse_hcim, parse_image, parse_pbuf, parse_sdoc0001_container,
        parse_sysp, parse_tebu_header, Flags, Line, Style, Te,
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
    fs::File,
    io::BufWriter,
    io::Write,
    path::{Path, PathBuf},
};

use super::font::ps::write_ls30_ps_bitmap;

mod ps;

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

pub struct Document<'a> {
    // Configuration
    print_driver: Option<PrintDriver>,
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
    fn chset<'b>(&'b self, pd: &PrintDriver, cset: u8) -> Option<&'b PSet<'a>> {
        match pd {
            PrintDriver::Editor => None,
            PrintDriver::Printer9 => self.chsets_p9[cset as usize].as_deref(),
            PrintDriver::Printer24 => self.chsets_p24[cset as usize].as_deref(),
            PrintDriver::Laser30 => self.chsets_l30[cset as usize].as_deref(),
        }
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
        if line.flags.contains(Flags::FLAG) && self.opt.format == Format::Html {
            println!("<F: {}>", line.extra);
        }

        if line.flags.contains(Flags::PARA) && self.opt.format == Format::Html {
            print!("<p>");
        }

        self.print_tebu_data(&line.data);

        if line.flags.contains(Flags::ALIG) && self.opt.format == Format::Html {
            print!("<A>");
        }

        if line.flags.contains(Flags::LINE) && self.opt.format == Format::Html {
            print!("<br>");
        }

        if self.opt.format == Format::Plain {
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
            (Ok(pset), FontKind::Needle9) => {
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
            all_pset &= self.load_cset_printer(index, cset_folder, name_ref, FontKind::Needle24);
            all_lset &= self.load_cset_printer(index, cset_folder, name_ref, FontKind::Laser30);
            all_p9 &= self.load_cset_printer(index, cset_folder, name_ref, FontKind::Needle9);
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
                PrintDriver::Printer9 => {
                    if !all_p9 {
                        println!("WARNING: Explicitly chosen 9-needle print-driver but not all fonts are available");
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
        } else if all_p9 {
            self.print_driver = Some(PrintDriver::Printer9);
        } else if all_eset {
            self.print_driver = Some(PrintDriver::Editor);
        } else {
            println!("No print-driver has all fonts available.");
        }
        self.chsets = charsets;
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
                Some(pd) => {
                    if let Some(eset) = self.chset(&pd, te.cset) {
                        let ch = &eset.chars[te.cval as usize];
                        let x = pd.scale_x(*x);
                        let y = pd.scale_y(y);
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

    fn output_print(&self, out_path: &Path) -> eyre::Result<()> {
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

    fn output_pdraw(&self) -> eyre::Result<()> {
        for page_text in &self.tebu {
            let mut pos = Pos::new(0, 0);
            for (skip, line) in &page_text.content {
                print_line_cmds(&line, *skip, &mut pos);
            }
        }
        Ok(())
    }

    fn output_console(&self) -> eyre::Result<()> {
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
        Ok(())
    }

    fn output_postscript(&self) -> eyre::Result<()> {
        if self.opt.out == Path::new("-") {
            println!("----------------------------- PostScript -----------------------------");
            let mut pw = PSWriter::new();
            self.output_ps_writer(&mut pw)?;
            println!("----------------------------------------------------------------------");
            Ok(())
        } else {
            let file = self.file.file_stem().unwrap();
            let out = {
                let mut buf = self.opt.out.join(file);
                buf.set_extension("ps");
                buf
            };
            let out_file = File::create(&out)?;
            let out_buf = BufWriter::new(out_file);
            let mut pw = PSWriter::from(out_buf);
            print!("Writing `{}` ...", out.display());
            self.output_ps_writer(&mut pw)?;
            println!(" Done!");
            Ok(())
        }
    }

    fn output_ps_writer(&self, pw: &mut PSWriter<impl Write>) -> eyre::Result<()> {
        let pd = self
            .print_driver
            .ok_or_else(|| eyre!("No printer type selected"))?;
        let (hdpi, vdpi) = pd.resolution();

        pw.write_magic()?;
        pw.write_meta_field("Creator", "Signum! Document Toolbox v0.3")?;
        let file_name = self.file.file_name().unwrap().to_string_lossy();
        pw.write_meta_field("Title", file_name.as_ref())?;
        //pw.write_meta_field("CreationDate", "Sun Sep 13 23:55:06 2020")?;
        pw.write_meta_field("Pages", &format!("{}", self.page_count))?;
        pw.write_meta_field("PageOrder", "Ascend")?;
        pw.write_meta_field("BoundingBox", "0 0 596 842")?;
        pw.write_meta_field("DocumentPaperSizes", "a4")?;
        pw.write_meta("EndComments")?;

        pw.write_meta_field("BeginProcSet", "signum.pro")?;
        pw.write_header_end()?;

        const DICT: &str = "SignumDict";
        const FONTS: [&str; 8] = ["Fa", "Fb", "Fc", "Fd", "Fe", "Ff", "Fg", "Fh"];
        prog_dict(pw, DICT)?;

        pw.write_meta("EndProcSet")?;
        pw.name(DICT)?;

        let mut use_matrix: [[usize; 128]; 8] = [[0; 128]; 8];

        for page in &self.tebu {
            for (_, line) in &page.content {
                for tw in &line.data {
                    use_matrix[tw.cset as usize][tw.cval as usize] += 1;
                }
            }
        }

        pw.begin(|pw| {
            pw.isize(39158280)?;
            pw.isize(55380996)?;
            pw.isize(1000)?;
            pw.isize(hdpi)?;
            pw.isize(vdpi)?;
            pw.bytes(b"hello.dvi")?;
            pw.crlf()?;
            pw.name("@start")?;
            for (i, use_matrix) in use_matrix.iter().enumerate() {
                match pd {
                    PrintDriver::Printer24 => {
                        if let Some(pset) = &self.chsets_p24[i] {
                            pw.write_comment(&format!("SignumBitmapFont: {}", &self.chsets[i]))?;
                            write_ls30_ps_bitmap(
                                FONTS[i],
                                &self.chsets[i],
                                pw,
                                pset,
                                Some(use_matrix),
                            )?;
                            pw.write_comment("EndSignumBitmapFont")?;
                        }
                    }
                    PrintDriver::Laser30 => {
                        if let Some(pset) = &self.chsets_l30[i] {
                            pw.write_comment(&format!("SignumBitmapFont: {}", &self.chsets[i]))?;
                            write_ls30_ps_bitmap(
                                FONTS[i],
                                &self.chsets[i],
                                pw,
                                pset,
                                Some(use_matrix),
                            )?;
                            pw.write_comment("EndSignumBitmapFont")?;
                        }
                    }
                    _ => {
                        println!("Print-Driver {:?} not yet supported", pd);
                    }
                }
            }

            Ok(())
        })?;
        pw.write_meta("EndProlog")?;

        pw.write_meta("BeginSetup")?;
        let feature = format!("*Resolution {}dpi", hdpi);
        pw.write_meta_field("Feature", &feature)?;

        pw.name(DICT)?;
        pw.begin(|pw| {
            pw.write_meta_field("BeginPaperSize", "a4")?;
            pw.lit("setpagedevice")?;
            pw.ps_where()?;
            pw.crlf()?;
            pw.seq(|pw| {
                pw.ps_pop()?;
                pw.dict(|pw| {
                    pw.lit("PageSize")?;
                    pw.arr(|pw| {
                        pw.isize(595)?;
                        pw.isize(842)
                    })
                })?;
                pw.ps_setpagedevice()
            })?;
            pw.crlf()?;
            pw.seq(|pw| {
                pw.lit("a4")?;
                pw.ps_where()?;
                pw.seq(|pw| {
                    pw.ps_pop()?;
                    pw.name("a4")
                })?;
                pw.ps_if()
            })?;
            pw.crlf()?;
            pw.ps_ifelse()?;
            pw.write_meta("EndPaperSize")?;
            Ok(())
        })?;
        pw.write_meta("EndSetup")?;

        let x_offset = self.opt.xoffset.unwrap_or(0);

        for (index, page) in self.tebu.iter().enumerate() {
            let page_info = self.pages[page.index as usize].as_ref().unwrap();
            let page_comment = format!("{} {}", page_info.log_pnr, page_info.phys_pnr);
            pw.write_meta_field("Page", &page_comment)?;

            pw.name(DICT)?;
            pw.begin(|pw| {
                let mut x: u16;
                let mut y: u16 = 0;
                let mut cset = 10;

                pw.isize(page_info.log_pnr as isize)?;
                pw.isize(index as isize)?;
                pw.name("bop")?;

                for (skip, line) in &page.content {
                    y += 1 + *skip;
                    x = 0;

                    let y_val = pd.scale_y(y) as isize;
                    for chr in &line.data {
                        // moveto
                        x += chr.offset;

                        if cset != chr.cset {
                            // select font a
                            cset = chr.cset;
                            pw.name(FONTS[chr.cset as usize])?;
                        }

                        let x_val = pd.scale_x(x) as isize + x_offset;
                        pw.isize(x_val)?;
                        pw.isize(y_val)?;
                        pw.name("a")?;

                        pw.bytes(&[chr.cval])?;
                        pw.name("p")?;
                    }
                }

                pw.name("eop")?;
                Ok(())
            })?;
        }
        pw.write_meta("Trailer")?;

        pw.ps_userdict()?;
        pw.lit("end-hook")?;
        pw.ps_known()?;
        pw.seq(|pw| pw.name("end-hook"))?;
        pw.ps_if()?;

        pw.write_meta("EOF")?;
        Ok(())
    }

    fn output(&self) -> eyre::Result<()> {
        match self.opt.format {
            Format::Html | Format::Plain => self.output_console(),
            Format::PostScript => self.output_postscript(),
            Format::PDraw => self.output_pdraw(),
            Format::Png => self.output_print(&self.opt.out),
            Format::DVIPSBitmapFont => panic!("Document can't be formatted as a font"),
            Format::CCITTT6 => panic!("Document can't be formatted as a font"),
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
