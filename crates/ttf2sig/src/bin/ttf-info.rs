use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::{Context, ContextCompat, OptionExt};
use signum::{
    chsets::{
        metrics::{FontMetrics, DEFAULT_FONT_SIZE},
        printer::PrinterKind,
    },
    image::{GrayImage, ImageFormat},
};
use ttf2sig::{glyph_index_vec, LigatureInfo};
use ttf_parser::{Face, GlyphId};

#[derive(Parser)]
/// Options for decoding an ATARI String
pub struct Opts {
    /// The file to convert
    font_file: PathBuf,

    /// The ligature to check
    #[clap(default_value = "ch")]
    ligature: String,

    #[clap(short, long, default_value = "0")]
    index: u32,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let opt: Opts = Opts::parse();
    let data = std::fs::read(&opt.font_file)?;

    let face = Face::parse(&data, opt.index)?;
    let l = LigatureInfo::new(&face);

    let lig = opt.ligature.chars().collect::<Vec<_>>();

    let glyphs = glyph_index_vec(&face, &lig).ok_or_eyre("Not all glyphs in font")?;
    let lig_glyph = l.find(&glyphs);
    println!("{:?} => {:?}", lig, lig_glyph);

    let font = fontdue::Font::from_bytes(
        &data[..],
        fontdue::FontSettings {
            collection_index: opt.index,
            scale: 40.0,
            load_substitutions: true,
        },
    )
    .expect("already ttf parsed");
    if let Some(gl) = lig_glyph {
        let img = _raster(&font, gl)?;
        _show(&img)?;
    }

    Ok(())
}

fn _raster(font: &fontdue::Font, g: GlyphId) -> color_eyre::Result<GrayImage> {
    let font_size = DEFAULT_FONT_SIZE;
    let pk = PrinterKind::Needle24;
    let fm = FontMetrics::new(pk, font_size);
    let px_per_em = fm.em_square_pixels();

    let (metrics, bitmap) = font.rasterize_indexed(g.0, px_per_em as f32);
    let inverted = bitmap.iter().copied().map(|c| 255 - c).collect();
    let img = GrayImage::from_vec(metrics.width as u32, metrics.height as u32, inverted)
        .context("image creation")?;
    Ok(img)
}

fn _show(img: &GrayImage) -> color_eyre::Result<()> {
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
