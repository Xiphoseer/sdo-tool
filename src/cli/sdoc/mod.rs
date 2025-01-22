use crate::cli::opt::{Format, Options};
use color_eyre::eyre::{self, eyre};
use image::ImageFormat;
use log::{debug, error, info};
use signum::{
    chsets::cache::{ChsetCache, DocumentFontCacheInfo, LocalFS, VFS},
    docs::{
        container::{parse_sdoc0001_container, Chunk},
        cset::CSet,
        hcim::{Hcim, ImageSite},
        header::parse_header,
        pbuf::{self, PBuf},
        sysp::SysP,
        tebu::{PageText, TeBu},
        DocumentInfo,
    },
    raster::Page,
    util::{Buf, FourCC},
};

use super::util;

mod console;
mod html;
mod imgseq;
pub mod pdf;
mod pdraw;
mod ps;
mod ps_proc;

pub struct Document<'a> {
    // Configuration
    opt: &'a Options,
    pages: Vec<Option<pbuf::Page>>,
    page_count: usize,
    // tebu
    pub(crate) tebu: TeBu,
    // hcim
    pub(crate) sites: Vec<ImageSite>,
}

impl<'a> Document<'a> {
    pub fn new(opt: &'a Options) -> Self {
        Document {
            opt,
            pages: vec![],
            page_count: 0,
            tebu: TeBu::default(),
            sites: vec![],
        }
    }

    pub fn text_pages(&self) -> &[PageText] {
        &self.tebu.pages
    }

    fn process_cset<FS: VFS>(
        &mut self,
        fc: &mut ChsetCache,
        fs: &FS,
        part: Buf<'_>,
    ) -> eyre::Result<DocumentFontCacheInfo> {
        info!("Loading 'cset' chunk");
        let charsets = util::load_chunk::<CSet>(part.0)?;
        info!("CHSETS: {:?}", charsets.names);

        let dfci = futures_lite::future::block_on(fc.load(fs, &charsets));
        Ok(dfci)
    }

    fn process_sysp(&mut self, part: Buf) -> eyre::Result<()> {
        info!("Loading 'sysp' chunk");
        let sysp = util::load_chunk::<SysP>(part.0)?;
        debug!("{:?}", sysp);
        Ok(())
    }

    fn process_pbuf(&mut self, part: Buf<'_>) -> eyre::Result<()> {
        info!("Loading 'pbuf' chunk");
        let pbuf = util::load_chunk::<PBuf>(part.0)?;

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
        self.tebu = util::load_chunk::<TeBu>(part.0)?;
        info!("Loaded text for {} page(s)!", self.page_count);
        Ok(())
    }

    fn process_hcim(&mut self, part: Buf) -> eyre::Result<Vec<(String, Page)>> {
        info!("Loading 'hcim' chunk");
        let hcim = util::load_chunk::<Hcim>(part.0)?;

        let out_img = self.opt.with_images.as_ref();
        if let Some(out_img) = out_img {
            std::fs::create_dir_all(out_img)?;
        }

        let images = hcim.decode_images();
        for (index, (key, page)) in images.iter().enumerate() {
            if let Some(out_img) = out_img {
                let name = format!("{:02}-{}.png", index, key);
                let path = out_img.join(name);
                let img = page.to_image();
                img.save_with_format(&path, ImageFormat::Png)?;
            }
        }

        self.sites = hcim.sites;

        Ok(images)
    }

    fn output(&self, fc: &ChsetCache, info: &DocumentInfo) -> eyre::Result<()> {
        let print = &info.fonts;
        let pd = print.print_driver(self.opt.print_driver);
        match self.opt.format {
            Format::Html => html::output_html(self, fc, print),
            Format::Plain => console::output_console(self, fc, print),
            Format::PostScript => ps::output_postscript(self, fc, print, pd),
            Format::PDraw => pdraw::output_pdraw(self),
            Format::Png => imgseq::output_print(self, fc, info, pd),
            Format::Pdf => pdf::output_pdf(self, fc, info, pd),
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

    pub fn process_sdoc<FS: VFS>(
        &mut self,
        input: &[u8],
        fs: &FS,
        fc: &mut ChsetCache,
    ) -> eyre::Result<DocumentInfo> {
        let sdoc = util::load(parse_sdoc0001_container, input)?;

        let mut dfci = None;
        let mut images = vec![];
        for Chunk { tag, buf } in sdoc.chunks {
            match tag {
                FourCC::_0001 => self.process_0001(buf),
                FourCC::_CSET => {
                    dfci = Some(self.process_cset(fc, fs, buf)?);
                    Ok(())
                }
                FourCC::_SYSP => self.process_sysp(buf),
                FourCC::_PBUF => self.process_pbuf(buf),
                FourCC::_TEBU => self.process_tebu(buf),
                FourCC::_HCIM => {
                    images = self.process_hcim(buf)?;
                    Ok(())
                }
                _ => {
                    info!("Found unknown chunk '{}' ({} bytes)", tag, buf.0.len());
                    Ok(())
                }
            }?;
        }

        let fonts = dfci.ok_or_else(|| eyre!("Document has no CSET chunk"))?;
        Ok(DocumentInfo::new(fonts, images))
    }

    pub fn text_buffer(&self) -> &TeBu {
        &self.tebu
    }
}

pub fn process_sdoc(input: &[u8], opt: Options) -> eyre::Result<()> {
    let mut document = Document::new(&opt);

    let folder = opt.file.parent().unwrap();
    let chsets_folder = folder.join(&opt.chsets_path);
    let fs = LocalFS::new(chsets_folder);
    let mut fc = ChsetCache::new();
    let di = document.process_sdoc(input, &fs, &mut fc)?;

    // Output the document
    document.output(&fc, &di)?;

    Ok(())
}
