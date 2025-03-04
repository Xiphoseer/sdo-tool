use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::{self, eyre, Context, ContextCompat};
use image::{GrayImage, ImageFormat};
use signum::chsets::printer::PrinterKind;

#[derive(Parser)]
/// Options for decoding an ATARI String
pub struct Opts {
    /// The file to convert
    file: PathBuf,
}

fn discretize(coverage: u8) -> u8 {
    if coverage >= 128 {
        0x00
    } else {
        0xFF
    }
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let opt: Opts = Opts::parse();

    // Read the font data.
    let font = std::fs::read(&opt.file)
        .wrap_err_with(|| format!("failed to read file '{}'", opt.file.display()))?;
    // Parse it into the font type.
    let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default())
        .map_err(|e| eyre!("Failed to load font: {}", e))?;
    // Rasterize and get the layout metrics for the letter 'g' at 17px.

    let size = PrinterKind::Needle24.line_height();

    let (metrics, bitmap) = font.rasterize('g', size as f32);
    let inverted = bitmap.iter().copied().map(discretize).collect();
    let img = GrayImage::from_vec(metrics.width as u32, metrics.height as u32, inverted)
        .context("image creation")?;

    let mut tmpfile =
        tempfile::NamedTempFile::with_suffix(".png").context("failed to create tempfile")?;
    img.write_to(&mut tmpfile, ImageFormat::Png)
        .expect("image to PDF");

    let mut child = std::process::Command::new("eog")
        .arg(tmpfile.path())
        .spawn()?;

    let _o = child.wait()?;

    Ok(())
}
