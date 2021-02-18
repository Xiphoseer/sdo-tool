use color_eyre::eyre::{self, eyre};
use image::ImageFormat;
use signum::{images::imc::parse_imc, raster::Page};
use std::path::{Path, PathBuf};

pub mod font;
pub mod opt;
pub mod sdoc;
mod util;

pub fn process_bimc(buffer: &[u8], file: &Path, out_path: Option<PathBuf>) -> eyre::Result<()> {
    let decoded = parse_imc(&buffer) //
        .map_err(|err| eyre!("Failed to parse: {}", err))?;

    let page = Page::from_screen(decoded);

    let out_path = if let Some(path) = out_path {
        path
    } else {
        file.with_extension("png")
    };

    let image = page.to_image();
    image.save_with_format(out_path, ImageFormat::Png)?;
    Ok(())
}
