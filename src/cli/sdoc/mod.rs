use std::path::Path;

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
        sysp::{parse_sysp, SysP},
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

pub struct Image {
    pub key: String,
    pub image: Page,
}

pub struct Document {
    // cset
    pub cset: [Option<String>; 8],
    pub chsets: [Option<usize>; 8],
    // sysp
    sysp: Option<SysP>,
    // pbuf
    pages: Vec<Option<pbuf::Page>>,
    page_count: usize,
    // tebu
    tebu: Vec<PageText>,
    // hcim
    pub(crate) images: Vec<Image>,
    pub(crate) sites: Vec<ImageSite>,
}

impl Document {
    pub fn page_count(&self) -> usize {
        self.tebu.len()
    }

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

    pub fn new() -> Self {
        Document {
            cset: [None, None, None, None, None, None, None, None],
            chsets: [None; 8],
            sysp: None,
            pages: vec![],
            page_count: 0,
            tebu: vec![],
            images: vec![],
            sites: vec![],
        }
    }

    fn process_cset<'x>(&mut self, part: Buf<'x>, fc: &mut ChsetCache) -> eyre::Result<()> {
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
        self.sysp = Some(sysp);
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

        let mut images = Vec::with_capacity(hcim.header.img_count as usize);

        for img in hcim.images {
            match parse_image(img.0) {
                Ok((_imgrest, im)) => {
                    debug!("Found image {:?}", im.key);
                    let page = Page::from_screen(im.image);
                    images.push(Image {
                        key: im.key.into_owned(),
                        image: page,
                    });
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

    fn output(&self, fc: &ChsetCache, opt: &Options) -> eyre::Result<()> {
        match opt.format {
            Format::Html => html::output_html(self, opt, fc),
            Format::Plain => console::output_console(self, opt, fc),
            Format::PostScript => ps::output_postscript(self, opt, fc),
            Format::PDraw => pdraw::output_pdraw(self),
            Format::Png => imgseq::output_print(self, opt, fc),
            Format::Pdf => pdf::output_pdf(self, opt, fc),
            Format::DviPsBitmapFont => {
                error!("Document export as PostScript font not supported!");
                Ok(())
            }
            Format::CcItt6 => {
                error!("Document export as CCITT-T6 (bitmap) not supported!");
                Ok(())
            }
            Format::Pbm => {
                error!("Document export as PBM (bitmap) not supported!");
                Ok(())
            }
            Format::Bdf => {
                error!("Document export as BDF (font) not supported!");
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

        for Chunk { tag, buf: part } in sdoc.chunks {
            match tag {
                "0001" => self.process_0001(part),
                "cset" => self.process_cset(part, fc),
                "sysp" => self.process_sysp(part),
                "pbuf" => self.process_pbuf(part),
                "tebu" => self.process_tebu(part),
                "hcim" => self.process_hcim(part),
                _ => {
                    info!("Found unknown chunk '{}' ({} bytes)", tag, part.0.len());
                    Ok(())
                }
            }?;
        }

        // Output rest
        if !rest.is_empty() {
            println!("remaining: {:#?}", Buf(rest));
        }

        Ok(())
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

fn output_images(doc: &Document, out_img: &Path) -> eyre::Result<()> {
    std::fs::create_dir_all(out_img)?;
    for (index, im) in doc.images.iter().enumerate() {
        let name = format!("{:02}-{}.png", index, im.key);
        let path = out_img.join(name);
        let img = im.image.to_image();
        img.save_with_format(&path, ImageFormat::Png)?;
    }
    Ok(())
}

pub fn process_sdoc(input: &[u8], opt: Options) -> eyre::Result<()> {
    let mut document = Document::new();

    let folder = opt.file.parent().unwrap();
    let chsets_folder = folder.join(&opt.chsets_path);
    let mut fc = ChsetCache::new(chsets_folder);

    document.process_sdoc(input, &mut fc)?;

    // Output images
    if let Some(out_img) = opt.with_images.as_ref() {
        output_images(&document, out_img)?;
    }

    // Output the document
    document.output(&fc, &opt)?;

    Ok(())
}
