use std::path::PathBuf;

use color_eyre::eyre::{self, eyre};
use image::ImageFormat;
use signum::{
    chsets::{cache::ChsetCache, FontKind},
    raster::render_doc_page,
};

use crate::cli::opt::Options;

use super::{Document, DocumentInfo};

pub fn output_print(
    doc: &Document,
    opt: &Options,
    fc: &ChsetCache,
    info: &DocumentInfo,
    pd: Option<FontKind>,
) -> eyre::Result<()> {
    let out_path: PathBuf = if let Some(path) = &opt.out {
        path.clone()
    } else {
        let dir = opt.file.with_extension("sdo.out");
        std::fs::create_dir(&dir)?;
        dir
    };

    let pd = pd.ok_or_else(|| eyre!("Print driver not set!"))?;

    for page_text in &doc.tebu.pages {
        let index = page_text.index as usize;
        let pbuf_entry = doc.pages[index].as_ref().unwrap();
        println!("{}", page_text.skip);
        if let Some(pages) = &opt.page {
            if !pages.contains(&(pbuf_entry.log_pnr as usize)) {
                continue;
            }
        }
        let page = render_doc_page(page_text, pbuf_entry, doc.image_sites(), info, pd, fc);
        let image = page.to_image();
        let file_name = format!("page-{}.png", pbuf_entry.log_pnr);
        println!("Saving {}", file_name);
        let page_path = out_path.join(&file_name);
        image.save_with_format(&page_path, ImageFormat::Png)?;
    }
    Ok(())
}
