use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::{Context, ContextCompat, OptionExt};
use signum::{
    chsets::{
        metrics::{FontMetrics, DEFAULT_FONT_SIZE},
        printer::PrinterKind,
    },
    image::{GenericImage, GrayImage, ImageFormat},
};
use ttf2sig::{glyph_index_vec, KerningInfo, LigatureInfo};
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
    let gpos = KerningInfo::new(&face);

    debug_kern(&face);

    let lig = opt.ligature.chars().collect::<Vec<_>>();

    let glyphs = glyph_index_vec(&face, &lig).ok_or_eyre("Not all glyphs in font")?;
    eprintln!("{lig:?} => {glyphs:?}");
    let lig_glyph = l.find(&glyphs);
    println!("{:?} => {:?}", lig, lig_glyph);

    for pair in glyphs.windows(2) {
        let (first, second) = (pair[0], pair[1]);
        // kern ttf-parser
        if let Some((v1, v2)) = gpos.find(first, second) {
            println!("v1: {v1:?}");
            println!("v2: {v2:?}");
        }
    }

    let font_size = DEFAULT_FONT_SIZE;
    let pk = PrinterKind::Needle24;
    let fm = FontMetrics::new(pk, font_size);
    let px = fm.em_square_pixels() as f32;

    let font = fontdue::Font::from_bytes(
        &data[..],
        fontdue::FontSettings {
            collection_index: opt.index,
            scale: 40.0,
            load_substitutions: true,
        },
    )
    .expect("already ttf parsed");

    for pair in lig.windows(2) {
        // kern fontdue
        let (a, b) = (pair[0], pair[1]);
        let kern = font.horizontal_kern(a, b, px);
        println!("kern {pair:?} => {:?}", kern);
    }

    if let Some(gl) = lig_glyph {
        let img = _raster(&font, gl, px)?;
        _show(&img)?;
    } else {
        let mut ymin = i32::MAX;
        let mut ymax = i32::MIN;
        let mut x = 0;
        let mut width = 0;
        let mut pos = vec![];
        for g in &glyphs {
            let m = font.metrics_indexed(g.0, px);
            println!("{m:?}");
            ymin = m.ymin.min(ymin);
            let y = m.ymin + (m.height as i32);
            ymax = y.max(ymax);
            width = (x + m.width).max(width);
            pos.push((x as u32, y));
            x += m.advance_width as usize;
        }
        let height = (ymax - ymin) as usize;
        let mut canvas = GrayImage::new(width as u32, height as u32);
        canvas.fill(0xFF);

        let mut pos_iter = pos.into_iter().map(|(x, y)| (x, (ymax - y) as u32));
        for g in glyphs {
            let img = _raster(&font, g, px)?;
            let (x, y) = pos_iter.next().expect("pos vec");
            canvas.copy_from(&img, x, y).unwrap();
        }
        _show(&canvas)?;
    }

    Ok(())
}

fn debug_kern(face: &Face<'_>) {
    if let Some(_kern) = face.tables().kern {
    } else {
        eprintln!("No kerning info (kern)");
    }

    if let Some(_kerx) = face.tables().kerx {
    } else {
        eprintln!("No extended kerning info (kerx)");
    }
}

fn _raster(font: &fontdue::Font, g: GlyphId, px: f32) -> color_eyre::Result<GrayImage> {
    let (metrics, bitmap) = font.rasterize_indexed(g.0, px);
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
