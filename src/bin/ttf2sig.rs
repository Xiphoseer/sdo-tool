use std::{io::BufWriter, path::PathBuf};

use clap::Parser;
use color_eyre::eyre::{self, eyre, Context, ContextCompat};
use image::{GrayImage, ImageFormat};
use signum::{
    chsets::{
        editor::{ESet, ECHAR_NULL},
        encoding::antikro,
        metrics::{FontMetrics, DEFAULT_FONT_SIZE},
        printer::{PSet, PSetChar, PrinterKind},
    },
    util::Buf,
};

#[derive(Parser)]
/// Options for decoding an ATARI String
pub struct Opts {
    /// The file to convert
    font_file: PathBuf,

    /// The directory to output
    out: PathBuf,

    /// Force overwrite existing files
    #[clap(short, long)]
    force: bool,

    /// The new name of the font
    #[clap(short, long)]
    name: Option<String>,

    /// Threshold at which to treat coverage as "on"
    #[clap(short, long, default_value = "128")]
    threshold: u8,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let opt: Opts = Opts::parse();

    // Read the font data.
    let font = std::fs::read(&opt.font_file)
        .wrap_err_with(|| format!("failed to read file '{}'", opt.font_file.display()))?;
    // Parse it into the font type.
    let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default())
        .map_err(|e| eyre!("Failed to load font: {}", e))?;

    let discretize = |c: u8| if c >= opt.threshold { 0x00 } else { 0xFF };

    let map = antikro::MAP;
    let name = opt.name.as_deref();
    let name = match name {
        Some(name) => name.to_owned(),
        None => derive_font_name(font.name().expect("missing font name")),
    };

    // Rasterize and get the layout metrics for the letter 'g' at 45px.
    let font_size = DEFAULT_FONT_SIZE;
    let pk = PrinterKind::Needle24;
    let fm = FontMetrics::new(pk, font_size);
    let px_per_em = fm.em_square_pixels();
    dbg!(px_per_em);

    let mut pset_chars = Vec::new();
    let mut eset_chars = Vec::new();
    for (index, c) in map.iter().copied().enumerate() {
        if c == '\0' || c == char::REPLACEMENT_CHARACTER || !font.has_glyph(c) {
            pset_chars.push(PSetChar::EMPTY);
            eset_chars.push(ECHAR_NULL);
            continue;
        }
        let (metrics, bitmap) = font.rasterize(c, px_per_em as f32);
        let inverted = bitmap.iter().copied().map(discretize).collect();
        let img = GrayImage::from_vec(metrics.width as u32, metrics.height as u32, inverted)
            .context("image creation")?;

        eprintln!("PCHAR: {:03}: {:?}", index, metrics);

        let bitmap = signum::raster::Page::from_image(&img, opt.threshold);
        let top = ((pk.baseline() as i32 - metrics.ymin) as usize - metrics.height) as u8;
        let pchar = PSetChar::from_page(top, bitmap).expect("failed to convert bitmap to char");
        pset_chars.push(pchar);
    }
    let pset = PSet {
        pk,
        header: Buf(&[0u8; 128]),
        chars: pset_chars,
    };
    let _eset = ESet {
        buf1: Buf(&[0u8; 128]),
        chars: eset_chars,
    };

    let outfile = opt.out.join(name).with_extension("P24");
    let outfile = match opt.force {
        true => std::fs::File::create(&outfile),
        false => std::fs::File::create_new(&outfile),
    }
    .wrap_err("failed to create output file")?;
    let mut writer = BufWriter::new(outfile);
    pset.write_to(&mut writer)?;

    Ok(())
}

fn derive_font_name(f: &str) -> String {
    let mut f: String = f
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    if f.len() > 8 {
        f = f.replace(['A', 'E', 'I', 'O', 'U'], "");
    }
    f
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
