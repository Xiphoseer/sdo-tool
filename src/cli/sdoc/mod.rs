use crate::cli::opt::{Format, Options};
use color_eyre::eyre::{self, eyre};
use image::ImageFormat;
use prettytable::{cell, format, row, Cell, Row, Table};
use signum::{
    chsets::{
        editor::ESet,
        printer::{PSet, PrinterKind},
        FontKind, UseMatrix,
    },
    docs::{
        container::{parse_sdoc0001_container, Chunk},
        cset::parse_cset,
        hcim::{parse_hcim, parse_image, ImageSite},
        header::parse_header,
        pbuf::{self, parse_pbuf},
        sysp::parse_sysp,
        tebu::{parse_page_text, parse_tebu_header, PageText},
    },
    nom::{multi::count, Finish},
    raster::Page,
    util::Buf,
};
use util::to_err_tree;

use super::{
    font::cache::{CSet, FontCache},
    util,
};

mod console;
mod imgseq;
pub mod pdf;
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
    //file: &'a Path,
    // cset
    pub cset: [Option<String>; 8],
    pub chsets: [Option<usize>; 8],
    // pbuf
    pages: Vec<Option<pbuf::Page>>,
    page_count: usize,
    // tebu
    tebu: Vec<PageText>,
    // hcim
    images: Vec<Page>,
    sites: Vec<ImageSite>,
}

impl<'a> Document<'a> {
    pub fn eset<'f>(&self, fc: &'f FontCache, cset: u8) -> Option<&'f ESet<'static>> {
        self.chsets[cset as usize].and_then(|index| fc.eset(index))
    }

    pub fn pset<'f>(
        &self,
        fc: &'f FontCache,
        cset: u8,
        pk: PrinterKind,
    ) -> Option<&'f PSet<'static>> {
        self.chsets[cset as usize].and_then(|index| fc.pset(pk, index))
    }

    pub fn cset<'f>(&self, fc: &'f FontCache, cset: u8) -> Option<&'f CSet> {
        self.chsets[cset as usize].and_then(|index| fc.cset(index))
    }

    pub fn use_matrix(&self) -> UseMatrix {
        let mut use_matrix = UseMatrix::new();

        for page in &self.tebu {
            for (_, line) in &page.content {
                for tw in &line.data {
                    let cval = tw.cval as usize;
                    let cset = tw.cset as usize;
                    use_matrix.csets[cset].chars[cval] += 1;
                }
            }
        }

        use_matrix
    }

    pub fn new(opt: &'a Options) -> Self {
        Document {
            opt,
            cset: [None, None, None, None, None, None, None, None],
            chsets: [None; 8],
            pages: vec![],
            page_count: 0,
            print_driver: opt.print_driver,
            tebu: vec![],
            images: vec![],
            sites: vec![],
        }
    }

    fn process_cset<'x>(&mut self, fc: &mut FontCache, part: Buf<'x>) -> eyre::Result<()> {
        let charsets = util::load(parse_cset, part.0)?;
        println!("'cset': {:?}", charsets);

        let mut all_eset = true;
        let mut all_p24 = true;
        let mut all_l30 = true;
        let mut all_p09 = true;
        for (index, name) in charsets.iter().enumerate() {
            if name.is_empty() {
                continue;
            }
            self.cset[index] = Some(name.to_string());
            let name_ref = name.as_ref();

            if let Some(cset_cache_index) = fc.load_cset(name_ref) {
                let cset = fc.cset(cset_cache_index).unwrap();
                self.chsets[index] = Some(cset_cache_index);
                all_eset &= cset.e24().is_some();
                all_p24 &= cset.p24().is_some();
                all_l30 &= cset.l30().is_some();
                all_p09 &= cset.p09().is_some();
            }
        }
        // Print info on which sets are available
        if all_eset {
            println!("Editor fonts available for all character sets");
        }
        if all_p24 {
            println!("Printer fonts (24-needle) available for all character sets");
        }
        if all_l30 {
            println!("Printer fonts (laser/30) available for all character sets");
        }
        if all_p09 {
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
                    if !all_p24 {
                        println!("WARNING: Explicitly chosen 24-needle print-driver but not all fonts are available");
                    }
                }
                FontKind::Printer(PrinterKind::Needle9) => {
                    if !all_p09 {
                        println!("WARNING: Explicitly chosen 9-needle print-driver but not all fonts are available");
                    }
                }
                FontKind::Printer(PrinterKind::Laser30) => {
                    if !all_l30 {
                        println!("WARNING: Explicitly chosen laser/30 print-driver but not all fonts are available");
                    }
                }
            }
        } else if all_l30 {
            self.print_driver = Some(FontKind::Printer(PrinterKind::Laser30));
        } else if all_p24 {
            self.print_driver = Some(FontKind::Printer(PrinterKind::Needle24));
        } else if all_p09 {
            self.print_driver = Some(FontKind::Printer(PrinterKind::Needle9));
        } else if all_eset {
            self.print_driver = Some(FontKind::Editor);
        } else {
            println!("No print-driver has all fonts available.");
        }
        Ok(())
    }

    fn process_sysp(&mut self, part: Buf) -> eyre::Result<()> {
        let sysp = util::load(parse_sysp, part.0)?;
        println!("'sysp': {:#?}", sysp);
        Ok(())
    }

    fn process_pbuf(&mut self, part: Buf<'_>) -> eyre::Result<()> {
        let pbuf = util::load(parse_pbuf, part.0)?;

        println!(
            "Page Buffer ('pbuf')\n  page_count: {}\n  kl: {}\n  first_page_nr: {}",
            pbuf.page_count, pbuf.kl, pbuf.first_page_nr
        );

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
        let (rest, hcim) = parse_hcim(part.0).finish().map_err(to_err_tree(part.0))?;
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
            "page", "pos_x", "pos_y", "site_w", "site_h", "[5]", "sel_x", "sel_y", "sel_w",
            "sel_h", "[A]", "[B]", "[C]", "img", "[E]", "[F]",
        ]);

        for isite in &hcim.sites {
            image_table.add_row(Row::new(vec![
                Cell::new(&format!("{}", isite.page)),
                Cell::new(&format!("{}", isite.site.x)),
                Cell::new(&format!("{}", isite.site.y)),
                Cell::new(&format!("{}", isite.site.w)),
                Cell::new(&format!("{}", isite.site.h)),
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

    fn output(&self, fc: &FontCache) -> eyre::Result<()> {
        match self.opt.format {
            Format::Html | Format::Plain => console::output_console(self, fc),
            Format::PostScript => ps::output_postscript(self, fc),
            Format::PDraw => pdraw::output_pdraw(self),
            Format::Png => imgseq::output_print(self, fc),
            Format::PDF => pdf::output_pdf(self, fc),
            Format::DVIPSBitmapFont | Format::CCITTT6 => {
                panic!("Document can't be formatted as a font")
            }
        }
    }

    pub fn process_0001(&mut self, part: Buf) -> eyre::Result<()> {
        let header = util::load(parse_header, part.0)?;
        println!("'0001':");
        println!("ctime: {}", header.ctime);
        println!("mtime: {}", header.mtime);
        Ok(())
    }

    pub fn process_sdoc(&mut self, input: &[u8], fc: &mut FontCache) -> eyre::Result<()> {
        let (rest, sdoc) = parse_sdoc0001_container(input)
            .finish()
            .map_err(|e| eyre!("Parse failed [{:?}]:\n{:?}", e.input, e.code))?;

        for Chunk { tag, buf } in sdoc.chunks {
            match tag {
                "0001" => self.process_0001(buf),
                "cset" => self.process_cset(fc, buf),
                "sysp" => self.process_sysp(buf),
                "pbuf" => self.process_pbuf(buf),
                "tebu" => self.process_tebu(buf),
                "hcim" => self.process_hcim(buf),
                _ => {
                    println!("'{}': {}", tag, buf.0.len());
                    Ok(())
                }
            }?;
        }

        if !rest.is_empty() {
            println!("remaining: {:#?}", Buf(rest));
        }

        Ok(())
    }
}

pub fn process_sdoc(input: &[u8], opt: Options) -> eyre::Result<()> {
    let mut document = Document::new(&opt);

    /*if opt.out != Path::new("-") {
        std::fs::create_dir_all(&opt.out)?;
    }*/

    let folder = opt.file.parent().unwrap();
    let chsets_folder = folder.join(&opt.chsets_path);
    let mut fc = FontCache::new(chsets_folder);
    document.process_sdoc(input, &mut fc)?;

    // Output the document
    document.output(&fc)?;

    Ok(())
}
