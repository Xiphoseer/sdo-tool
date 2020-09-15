use crate::{images::imc::parse_imc, print::Page};
use anyhow::anyhow;
use image::ImageFormat;
use std::path::PathBuf;

mod font;
pub mod keyboard;
pub mod ps;
pub mod sdoc;

pub use font::{process_eset, process_ls30, process_ps24};

pub fn process_bimc(buffer: &[u8], out_path: PathBuf) -> anyhow::Result<()> {
    let decoded = parse_imc(&buffer) //
        .map_err(|err| anyhow!("Failed to parse: {}", err))?;

    let page = Page::from_screen(decoded);

    let image = page.to_image();
    image.save_with_format(out_path, ImageFormat::Png)?;
    Ok(())
}
