use std::{
    convert::TryInto,
    io::BufWriter,
    num::TryFromIntError,
    path::{Path, PathBuf},
};

use clap::Parser;
use color_eyre::eyre::{self, eyre, Context, ContextCompat};
use signum::{
    chsets::{
        editor::{EChar, ESet, ECHAR_NULL},
        metrics::{FontMetrics, DEFAULT_FONT_SIZE},
        printer::{PSet, PSetChar, PrinterKind},
        FontKind,
        FontKind::Editor,
    },
    image::{GrayImage, ImageFormat},
    util::{Buf, FileFormatKind},
};
use ttf2sig::{glyph_index_vec, LigatureInfo};
use ttf_parser::GlyphId;

#[derive(Parser)]
/// Options for decoding an ATARI String
pub struct Opts {
    /// The file to convert
    font_file: PathBuf,

    /// The directory to output
    out: PathBuf,

    #[clap(short, long, default_value = "ANTIKRO")]
    mapping: String,

    /// Force overwrite existing files
    #[clap(short, long)]
    force: bool,

    /// Index (of a multi-font file)
    #[clap(short, long, default_value = "0")]
    index: u32,

    /// The new name of the font
    #[clap(short, long)]
    name: Option<String>,

    /// Threshold at which to treat coverage as "on"
    #[clap(short, long, default_value = "170")]
    threshold: u8,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let opt: Opts = Opts::parse();

    // Read the font data.
    let font = std::fs::read(&opt.font_file)
        .wrap_err_with(|| format!("failed to read file '{}'", opt.font_file.display()))?;
    // Raw parse for ligature info
    let face = ttf_parser::Face::parse(&font, opt.index)?;
    let ligatures = LigatureInfo::new(&face);

    // Parse it into the font type.
    let font_settings = fontdue::FontSettings {
        collection_index: opt.index,
        load_substitutions: true,
        ..Default::default()
    };
    let font = fontdue::Font::from_bytes(&font[..], font_settings)
        .map_err(|e| eyre!("Failed to load font: {}", e))?;

    let threshold = opt.threshold;

    let map = sdo_fonts::mappings::lookup(&opt.mapping)
        .ok_or_else(|| eyre!("Unknown mapping {:?}", opt.mapping))?;
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

    let editor_font_metrics = FontMetrics::new(FontKind::Editor, font_size);
    let e_px_per_em = editor_font_metrics.em_square_pixels();

    let mut pset_chars = Vec::new();
    let mut eset_chars = Vec::new();
    for (index, c) in map.chars().enumerate() {
        let glyph_index = find_glyph(&face, &ligatures, c);

        let Some(glyph_id) = glyph_index else {
            pset_chars.push(PSetChar::EMPTY);
            eset_chars.push(ECHAR_NULL);
            continue;
        };

        println!("Converting 0x{:02x}: {:?} => {}", index, c, glyph_id.0); //  (U+{:04X})

        let (p_metrics, p_bitmap) = rasterize(threshold, &font, px_per_em, None, glyph_id)?;
        let (e_metrics, e_bitmap) = rasterize(threshold, &font, e_px_per_em, Some(16), glyph_id)?;

        pset_chars.push(make_pchar(pk, p_metrics, p_bitmap));
        eset_chars.push(make_echar(e_metrics, e_bitmap).unwrap());
    }
    let pset = PSet {
        pk,
        header: Buf(&[0u8; 128]),
        chars: pset_chars,
    };
    let eset = ESet {
        buf1: Buf(&[0u8; 128]),
        chars: eset_chars,
    };

    let out_dir = &opt.out;
    write_pset(&name, pset, out_dir, opt.force)?;
    write_eset(&name, eset, out_dir, opt.force)?;

    Ok(())
}

fn find_glyph(
    face: &ttf_parser::Face<'_>,
    ligatures: &LigatureInfo<'_>,
    c: &[char],
) -> Option<GlyphId> {
    match c {
        [] => None,
        [c] => face.glyph_index(*c),
        _ => {
            let glyph_ids = glyph_index_vec(face, c)?;
            ligatures.find(&glyph_ids)
        }
    }
}

fn write_pset(name: &str, pset: PSet<'_>, out_dir: &Path, force: bool) -> Result<(), eyre::Error> {
    let outfile = out_dir.join(name).with_extension(pset.pk.extension());
    let mut writer = create_output_file(&outfile, force)?;
    pset.write_to(&mut writer)?;
    eprintln!("Wrote {}", outfile.display());
    Ok(())
}

fn write_eset(name: &str, pset: ESet<'_>, out_dir: &Path, force: bool) -> Result<(), eyre::Error> {
    let outfile = out_dir.join(name).with_extension(Editor.extension());
    let mut writer = create_output_file(&outfile, force)?;
    pset.write_to(&mut writer)?;
    eprintln!("Wrote {}", outfile.display());
    Ok(())
}

fn create_output_file(path: &Path, force: bool) -> Result<BufWriter<std::fs::File>, eyre::Error> {
    match force {
        true => std::fs::File::create(path),
        false => std::fs::File::create_new(path),
    }
    .wrap_err("failed to create output file")
    .map(BufWriter::new)
}

fn make_echar(
    metrics: fontdue::Metrics,
    bitmap: signum::raster::Page,
) -> Result<EChar<'static>, TryFromIntError> {
    let width = metrics.advance_width.round() as u8;
    let height = bitmap.bit_height().try_into()?;
    let top = ((FontKind::Editor.baseline() as i32 - metrics.ymin) as usize - metrics.height) as u8;
    Ok(EChar::new_owned(width, height, top, bitmap.into_vec()).unwrap())
}

fn make_pchar(
    pk: PrinterKind,
    metrics: fontdue::Metrics,
    bitmap: signum::raster::Page,
) -> PSetChar<'static> {
    let top = ((pk.baseline() as i32 - metrics.ymin) as usize - metrics.height) as u8;
    let pchar = PSetChar::from_page(top, bitmap).expect("failed to convert bitmap to char");
    pchar
}

fn rasterize(
    threshold: u8,
    font: &fontdue::Font,
    px_per_em: u32,
    req_width: Option<u8>,
    g: GlyphId,
) -> Result<(fontdue::Metrics, signum::raster::Page), eyre::Error> {
    let (metrics, bitmap) = font.rasterize_indexed(g.0, px_per_em as f32);
    let inverted = bitmap.iter().copied().map(|c| 255 - c).collect();
    let mut img = GrayImage::from_vec(metrics.width as u32, metrics.height as u32, inverted)
        .context("image creation")?;
    let mut lpad = metrics.xmin.max(0); // FIXME: not ideal, but we can't draw left of origin
    if let Some(max) = req_width {
        if img.width() > max as u32 {
            println!(
                "WARN: editor font limited to {max} width, is {}, truncating",
                img.width()
            );
            let mut buf = Vec::new();
            for row in img.rows() {
                for pixel in row.take(max as usize) {
                    buf.push(pixel.0[0]);
                }
            }
            img = GrayImage::from_vec(max.into(), metrics.height as u32, buf).unwrap();
            lpad = 0;
        }
    }
    let covered = lpad as u8 + metrics.width as u8;
    let rpad = match req_width {
        Some(w) if w > covered => w - covered,
        _ => 0,
    };
    let bitmap = signum::raster::Page::from_image(&img, threshold, (lpad as _, rpad));
    Ok((metrics, bitmap))
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
