use std::fmt::{self, Write};

use color_eyre::eyre::{self, eyre};
use image::ImageFormat;
use log::info;
use signum::{
    images::imc::{parse_imc, MonochromeScreen},
    raster::Page,
};

use super::opt::{Format, Options};

pub fn write_pbm(image: MonochromeScreen, out: &mut String) -> fmt::Result {
    writeln!(out, "P1 640 400")?;
    for line in image.into_inner().chunks(8) {
        for byte in line {
            write!(out, "{:08b}", byte)?;
        }
        writeln!(out)?;
    }
    Ok(())
}

pub fn process_bimc(buffer: &[u8], opt: Options) -> eyre::Result<()> {
    info!("Found Signum! image (bimc)");
    let decoded = parse_imc(&buffer) //
        .map_err(|err| eyre!("Failed to parse: {}", err))?;

    let file = opt.file;
    match &opt.format {
        Format::Png => {
            let out_path = opt.out.unwrap_or_else(|| file.with_extension("png"));
            let page = Page::from_screen(decoded);
            let image = page.to_image();

            image.save_with_format(&out_path, ImageFormat::Png)?;
            info!("Saved image as '{}'", out_path.display());
        }
        Format::Pbm => {
            let mut out = String::new();
            write_pbm(decoded, &mut out).unwrap();

            let out_path = opt.out.unwrap_or_else(|| file.with_extension("pbm"));
            std::fs::write(&out_path, out)?;
            info!("Saved image as '{}'", out_path.display());
        }
        _ => {
            info!("Use `--format png` or `--format pbm` to convert");
        }
    }

    Ok(())
}
