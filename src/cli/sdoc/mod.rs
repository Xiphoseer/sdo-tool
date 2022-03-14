use crate::cli::opt::{Format, Options};
use color_eyre::eyre::{self, eyre};
use image::ImageFormat;
use log::{debug, error, info};
use signum::{
    chsets::{
        cache::{CSet, ChsetCache},
        editor::ESet,
        printer::{PSet, PrinterKind},
        UseMatrix,
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

use super::util;

mod console;
mod html;
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
    opt: &'a Options,
    // cset
    pub cset: [Option<String>; 8],
    pub chsets: [Option<usize>; 8],
    // pbuf
    pages: Vec<Option<pbuf::Page>>,
    page_count: usize,
    // tebu
    tebu: Vec<PageText>,
    // hcim
    pub(crate) images: Vec<Page>,
    pub(crate) sites: Vec<ImageSite>,
}

impl<'a> Document<'a> {
    pub fn eset<'f>(&self, fc: &'f ChsetCache, cset: u8) -> Option<&'f ESet<'static>> {
        self.chsets[cset as usize].and_then(|index| fc.eset(index))
    }

    pub fn pset<'f>(
        &self,
        fc: &'f ChsetCache,
        cset: u8,
        pk: PrinterKind,
    ) -> Option<&'f PSet<'static>> {
        self.chsets[cset as usize].and_then(|index| fc.pset(pk, index))
    }

    pub fn cset<'f>(&self, fc: &'f ChsetCache, cset: u8) -> Option<&'f CSet> {
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
            //print_driver: opt.print_driver,
            tebu: vec![],
            images: vec![],
            sites: vec![],
        }
    }

    fn process_cset<'x>(&mut self, fc: &mut ChsetCache, part: Buf<'x>) -> eyre::Result<()> {
        info!("Loading 'cset' chunk");
        let charsets = util::load(parse_cset, part.0)?;
        info!("CHSETS: {:?}", charsets);

        for (index, name) in charsets.iter().enumerate() {
            if name.is_empty() {
                continue;
            }
            self.cset[index] = Some(name.to_string());
            self.chsets[index] = fc.load_cset(name.as_ref());
        }
        Ok(())
    }

    fn process_sysp(&mut self, part: Buf) -> eyre::Result<()> {
        info!("Loading 'sysp' chunk");
        let sysp = util::load(parse_sysp, part.0)?;
        debug!("{:?}", sysp);
        Ok(())
    }

    fn process_pbuf(&mut self, part: Buf<'_>) -> eyre::Result<()> {
        info!("Loading 'pbuf' chunk");
        let pbuf = util::load(parse_pbuf, part.0)?;

        debug!(
            "PageBuffer {{ page_count: {}, elem_len: {}, first_page_nr: {} }}",
            pbuf.page_count, pbuf.elem_len, pbuf.first_page_nr
        );

        self.pages = pbuf.pages.into_iter().map(|f| f.map(|(p, _b)| p)).collect();
        self.page_count = pbuf.page_count as usize;

        info!("Loaded page table with {} entries", self.page_count);

        Ok(())
    }

    fn process_tebu(&mut self, part: Buf) -> eyre::Result<()> {
        info!("Loading 'tebu' chunk");
        let (rest, tebu_header) = parse_tebu_header(part.0).unwrap();
        debug!("{:?}", tebu_header);

        let (rest, tebu) = match count(parse_page_text, self.page_count)(rest) {
            Ok(r) => r,
            Err(e) => {
                return Err(eyre!("Failed to process pages: {}", e));
            }
        };
        self.tebu = tebu;
        if !rest.is_empty() {
            debug!("rest(tebu): {:#?}", Buf(rest));
        }
        info!("Loaded text for {} page(s)!", self.page_count);
        Ok(())
    }

    fn process_hcim(&mut self, part: Buf) -> eyre::Result<()> {
        info!("Loading 'hcim' chunk");
        let (rest, hcim) = parse_hcim(part.0).finish().map_err(to_err_tree(part.0))?;

        debug!("{:?}", hcim.header);

        let out_img = self.opt.with_images.as_ref();
        if let Some(out_img) = out_img {
            std::fs::create_dir_all(out_img)?;
        }

        let mut images = Vec::with_capacity(hcim.header.img_count as usize);

        for (index, img) in hcim.images.iter().enumerate() {
            //println!("image[{}]:", index);
            match parse_image(img.0) {
                Ok((_imgrest, im)) => {
                    debug!("Found image {:?}", im.key);
                    //println!("{:#?}", im.bytes);
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
                    error!("Error: {}", e);
                }
            }
        }
        info!("Found {} image(s)", images.len());

        self.images = images;
        self.sites = hcim.sites;

        if !rest.is_empty() {
            println!("{:#?}", Buf(rest));
        }

        Ok(())
    }

    fn output(&self, fc: &ChsetCache) -> eyre::Result<()> {
        let print_driver = fc.print_driver(self.opt.print_driver)?;
        match self.opt.format {
            Format::Html => html::output_html(self, fc),
            Format::Plain => console::output_console(self, fc),
            Format::PostScript => ps::output_postscript(self, fc),
            Format::PDraw => pdraw::output_pdraw(self),
            Format::Png => imgseq::output_print(self, fc, print_driver),
            Format::Pdf => pdf::output_pdf(self, fc),
            Format::DviPsBitmapFont | Format::CcItt6 => {
                error!("Document can't be formatted as a font");
                Ok(())
            }
            Format::Pbm => {
                error!("Document export as PBM not supported!");
                Ok(())
            }
        }
    }

    pub fn process_0001(&mut self, part: Buf) -> eyre::Result<()> {
        let header = util::load(parse_header, part.0)?;
        info!("Loading '0001' chunk");
        info!("File created: {}", header.ctime);
        info!("File modified: {}", header.mtime);
        Ok(())
    }

    pub fn process_sdoc(&mut self, input: &[u8], fc: &mut ChsetCache) -> eyre::Result<()> {
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
                    info!("Found unknown chunk '{}' ({} bytes)", tag, buf.0.len());
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

    let folder = opt.file.parent().unwrap();
    let chsets_folder = folder.join(&opt.chsets_path);
    let mut fc = ChsetCache::new(chsets_folder);
    document.process_sdoc(input, &mut fc)?;

    // Output the document
    document.output(&fc)?;

    Ok(())
}
