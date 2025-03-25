use std::path::Path;

use crate::cli::opt::{Format, Options};
use color_eyre::eyre::{self, eyre};
use image::ImageFormat;
use log::{debug, error, info};
use signum::{
    chsets::cache::ChsetCache,
    docs::{
        container::{parse_sdoc0001_container, Chunk},
        cset::CSet,
        hcim::{Hcim, ImageSite},
        header::parse_header,
        pbuf::{self, PBuf},
        sysp::SysP,
        tebu::{PageText, TeBu},
        v3::parse_sdoc_v3,
        DocumentInfo,
    },
    util::{Buf, FourCC, LocalFS, VFS},
};

use super::util;

mod console;
mod html;
mod imgseq;
pub mod pdf;
mod pdraw;
mod ps;
mod ps_proc;

#[derive(Default)]
pub struct Document {
    pages: Vec<Option<pbuf::Page>>,
    page_count: usize,
    pub(crate) cset: Option<CSet<'static>>,
    pub(crate) sysp: Option<SysP>,
    // tebu
    pub(crate) tebu: TeBu,
    // hcim
    pub(crate) hcim: Option<Hcim<'static>>,
}

impl Document {
    pub fn new() -> Self {
        Document::default()
    }

    pub fn text_pages(&self) -> &[PageText] {
        &self.tebu.pages
    }

    pub fn image_sites(&self) -> &[ImageSite] {
        self.hcim
            .as_ref()
            .map(|hcim| &hcim.sites[..])
            .unwrap_or(&[])
    }

    fn process_cset(&mut self, part: Buf<'_>) -> eyre::Result<()> {
        debug!("Loading 'cset' chunk");
        let charsets = util::load_chunk::<CSet>(part.0)?;
        info!("CHSETS: {:?}", charsets.names);
        self.cset = Some(charsets.into_owned());
        Ok(())
    }

    fn process_sysp(&mut self, part: Buf) -> eyre::Result<()> {
        debug!("Loading 'sysp' chunk");
        let sysp = util::load_chunk::<SysP>(part.0)?;
        debug!("{:#?}", sysp);
        self.sysp = Some(sysp);
        Ok(())
    }

    fn process_pbuf(&mut self, part: Buf<'_>) -> eyre::Result<()> {
        debug!("Loading 'pbuf' chunk");
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
        debug!("Loading 'tebu' chunk");
        self.tebu = util::load_chunk::<TeBu>(part.0)?;
        info!("Loaded text for {} page(s)!", self.page_count);
        Ok(())
    }

    fn process_hcim(&mut self, part: Buf) -> eyre::Result<()> {
        debug!("Loading 'hcim' chunk");
        let hcim = util::load_chunk::<Hcim>(part.0)?;
        self.hcim = Some(hcim.into_owned());
        Ok(())
    }

    fn output(&self, fc: &ChsetCache, info: &DocumentInfo, opt: &Options) -> eyre::Result<()> {
        let print = &info.fonts;
        let pd = print.print_driver(opt.print_driver);
        match opt.format {
            Format::Html => html::output_html(self, opt, fc, print),
            Format::Plain => console::output_console(self, opt, fc, print),
            Format::PostScript => ps::output_postscript(self, opt, fc, print, pd),
            Format::PDraw => pdraw::output_pdraw(self),
            Format::Png => imgseq::output_print(self, opt, fc, info, pd),
            Format::Pdf => pdf::output_pdf(self, opt, fc, info, pd),
            Format::DviPsBitmapFont | Format::CcItt6 => {
                error!("Document can't be formatted as a font");
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
        debug!("Loading '0001' chunk");
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

        for Chunk { tag, buf } in sdoc.chunks {
            match tag {
                FourCC::_0001 => self.process_0001(buf),
                FourCC::_CSET => self.process_cset(buf),
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

        let cset = self
            .cset
            .as_ref()
            .ok_or_else(|| eyre!("Document has no CSET chunk"))?;
        let fonts = futures_lite::future::block_on(fc.load(fs, cset));
        let images = self
            .hcim
            .as_ref()
            .map(|hcim| hcim.decode_images())
            .unwrap_or_default();
        Ok(DocumentInfo::new(fonts, images))
    }

    pub fn text_buffer(&self) -> &TeBu {
        &self.tebu
    }
}

fn output_images(doc: &DocumentInfo, out_img: &Path) -> eyre::Result<()> {
    std::fs::create_dir_all(out_img)?;
    for (index, im) in doc.images().enumerate() {
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
    let fs = LocalFS::new(chsets_folder);
    let mut fc = ChsetCache::new();
    let di = document.process_sdoc(input, &fs, &mut fc)?;

    // Output images
    if let Some(out_img) = opt.with_images.as_ref() {
        output_images(&di, out_img)?;
    }

    // Output the document
    document.output(&fc, &di, &opt)?;

    Ok(())
}

pub fn process_sdoc_v3(input: &[u8], _opt: Options) -> eyre::Result<()> {
    let (_, sdoc) = util::load_partial(parse_sdoc_v3, input)?;
    log::info!("File Pointers: {:?}", sdoc.flptrs01());
    for (i, name) in sdoc.foused01().fonts() {
        log::info!("Font Used: {:>3} {:?}", i, name);
    }
    Ok(())
}
