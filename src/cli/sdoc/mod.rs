use crate::cli::opt::{Format, Options};
use color_eyre::eyre::{self, eyre};
use image::ImageFormat;
use log::{debug, error, info};
use nom_supreme::error::ErrorTree;
use signum::{
    chsets::{
        cache::{ChsetCache, DocumentFontCacheInfo},
        UseMatrix,
    },
    docs::{
        self,
        container::{parse_sdoc0001_container, Chunk},
        cset::{self},
        hcim::{parse_hcim, parse_image, ImageSite},
        header::parse_header,
        pbuf::{self, parse_pbuf},
        sysp::parse_sysp,
        tebu::{parse_page_text, parse_tebu_header, PageText},
    },
    nom::{multi::count, Finish},
    raster::Page,
    util::{Buf, FourCC},
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
    pages: Vec<Option<pbuf::Page>>,
    page_count: usize,
    // tebu
    tebu: Vec<PageText>,
    // hcim
    pub(crate) images: Vec<Page>,
    pub(crate) sites: Vec<ImageSite>,
}

pub struct DocumentInfo {
    pub fonts: DocumentFontCacheInfo,
}

impl<'a> Document<'a> {
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
            pages: vec![],
            page_count: 0,
            tebu: vec![],
            images: vec![],
            sites: vec![],
        }
    }

    fn process_cset(
        &mut self,
        fc: &mut ChsetCache,
        part: Buf<'_>,
    ) -> eyre::Result<DocumentFontCacheInfo> {
        info!("Loading 'cset' chunk");
        let charsets = util::load(<cset::CSet as docs::Chunk>::parse, part.0)?;
        info!("CHSETS: {:?}", charsets.names);

        Ok(fc.load(&charsets))
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
        let (rest, tebu_header) = parse_tebu_header::<ErrorTree<&[u8]>>(part.0).unwrap();
        debug!("{:?}", tebu_header);

        let (rest, tebu) = match count(parse_page_text::<ErrorTree<&[u8]>>, self.page_count)(rest) {
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
            match parse_image(img.0) {
                Ok((_imgrest, im)) => {
                    debug!("Found image {:?}", im.key);
                    let page = Page::from(im.image);
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

    fn output(&self, fc: &ChsetCache, print: &DocumentFontCacheInfo) -> eyre::Result<()> {
        let pd = print.print_driver(self.opt.print_driver);
        match self.opt.format {
            Format::Html => html::output_html(self, fc, print),
            Format::Plain => console::output_console(self, fc, print),
            Format::PostScript => ps::output_postscript(self, fc, print, pd),
            Format::PDraw => pdraw::output_pdraw(self),
            Format::Png => imgseq::output_print(self, fc, print, pd),
            Format::Pdf => pdf::output_pdf(self, fc, print, pd),
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

    pub fn process_sdoc(
        &mut self,
        input: &[u8],
        fc: &mut ChsetCache,
    ) -> eyre::Result<DocumentInfo> {
        let (rest, sdoc) = parse_sdoc0001_container(input)
            .finish()
            .map_err(|e| eyre!("Parse failed [{:?}]:\n{:?}", e.input, e.code))?;

        let mut dfci = None;
        for Chunk { tag, buf } in sdoc.chunks {
            match tag {
                FourCC::_0001 => self.process_0001(buf),
                FourCC::_CSET => {
                    dfci = Some(self.process_cset(fc, buf)?);
                    Ok(())
                }
                FourCC::_SYSP => self.process_sysp(buf),
                FourCC::_PBUF => self.process_pbuf(buf),
                FourCC::_TEBU => self.process_tebu(buf),
                FourCC::_HCIM => self.process_hcim(buf),
                _ => {
                    info!("Found unknown chunk '{}' ({} bytes)", tag, buf.0.len());
                    Ok(())
                }
            }?;
        }

        if !rest.is_empty() {
            println!("remaining: {:#?}", Buf(rest));
        }

        let fonts = dfci.ok_or_else(|| eyre!("Document has no CSET chunk"))?;
        Ok(DocumentInfo { fonts })
    }
}

pub fn process_sdoc(input: &[u8], opt: Options) -> eyre::Result<()> {
    let mut document = Document::new(&opt);

    let folder = opt.file.parent().unwrap();
    let chsets_folder = folder.join(&opt.chsets_path);
    let mut fc = ChsetCache::new(chsets_folder);
    let di = document.process_sdoc(input, &mut fc)?;

    // Output the document
    document.output(&fc, &di.fonts)?;

    Ok(())
}
