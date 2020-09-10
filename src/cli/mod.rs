use crate::{images::imc::parse_imc, print::Page};
use anyhow::anyhow;
use image::ImageFormat;
use std::path::PathBuf;

mod font;
pub mod keyboard;
pub mod sdoc;

pub use font::{process_eset, process_ls30, process_ps24};

pub fn process_bimc(buffer: &[u8], out: Option<PathBuf>) -> anyhow::Result<()> {
    let decoded = parse_imc(&buffer) //
        .map_err(|err| anyhow!("Failed to parse: {}", err))?;

    let page = Page::from_screen(decoded);

    if let Some(out_path) = out {
        let image = page.to_image();
        image.save_with_format(out_path, ImageFormat::Png)?;
    } else {
        println!("Decoded image sucessfully, to store it as PNG, pass `--out <PATH>`");
    }
    Ok(())
}
