use std::{fs::File, io::BufWriter, path::Path};

use crate::cli::opt::Options;

use super::{Document, DocumentInfo};
use color_eyre::eyre::{self, eyre, OptionExt};
use log::info;
use sdo_pdf::{generate_pdf, MetaInfo, Pdf};
use signum::{
    chsets::{cache::ChsetCache, FontKind},
    docs::{hcim::ImageSite, pbuf, sysp::SysP, tebu::PageText, GenerationContext, Overrides},
};

pub struct GenCtx<'a> {
    di: &'a DocumentInfo,
    image_sites: &'a [ImageSite],
    text_pages: &'a [PageText],
    pages: &'a [Option<pbuf::Page>],
    sysp: &'a SysP,
}

impl<'a> GenCtx<'a> {
    pub fn new(doc: &'a Document, di: &'a DocumentInfo) -> Self {
        Self {
            di,
            image_sites: doc.image_sites(),
            text_pages: doc.text_pages(),
            pages: &doc.pages[..],
            sysp: doc.sysp.as_ref().expect("missing sysp chunk"),
        }
    }
}

impl GenerationContext for GenCtx<'_> {
    fn image_sites(&self) -> &[ImageSite] {
        self.image_sites
    }

    fn document_info(&self) -> &DocumentInfo {
        self.di
    }

    fn text_pages(&self) -> &[PageText] {
        self.text_pages
    }

    fn page_at(&self, index: usize) -> Option<&pbuf::Page> {
        self.pages[index].as_ref()
    }

    fn sysp(&self) -> &signum::docs::sysp::SysP {
        self.sysp
    }
}

fn doc_meta(opt: &Options) -> eyre::Result<(MetaInfo, Overrides)> {
    let meta = opt.meta()?;
    let file_name = opt
        .file
        .file_name()
        .ok_or_eyre("expect file to have name")?;
    let file_name = file_name
        .to_str()
        .ok_or_eyre("File name contains invalid characters")?;
    let info = meta.pdf_meta_info(file_name);
    let overrides = meta.to_overrides();
    Ok((info, overrides))
}

pub fn output_pdf(
    doc: &Document,
    opt: &Options,
    fc: &ChsetCache,
    di: &DocumentInfo,
    pd: Option<FontKind>,
) -> eyre::Result<()> {
    let pk = match pd.ok_or_else(|| eyre!("No printer type selected"))? {
        FontKind::Printer(pk) => Ok(pk),
        FontKind::Editor => Err(eyre!("Editor fonts are not currently supported")),
    }?;
    let (meta, overrides) = doc_meta(opt)?;
    let out_path = opt.out.as_deref();

    let pdf = generate_pdf(fc, pk, &meta, &overrides, &GenCtx::new(doc, di))?;
    handle_out(out_path, &opt.file, pdf)?;
    Ok(())
}

pub fn handle_out(out: Option<&Path>, file: &Path, hnd: Pdf) -> eyre::Result<()> {
    if out == Some(Path::new("-")) {
        println!("----------------------------- PDF -----------------------------");
        let stdout = std::io::stdout();
        let mut stdolock = stdout.lock();
        hnd.write(&mut stdolock)?;
        println!("---------------------------------------------------------------");
        Ok(())
    } else {
        let out = out.unwrap_or_else(|| file.parent().unwrap());
        let file = file.file_stem().unwrap();
        let out = {
            let mut buf = out.join(file);
            buf.set_extension("pdf");
            buf
        };
        let out_file = File::create(&out)?;
        let mut out_buf = BufWriter::new(out_file);
        info!("Writing `{}` ...", out.display());
        hnd.write(&mut out_buf)?;
        info!("Done!");
        Ok(())
    }
}
