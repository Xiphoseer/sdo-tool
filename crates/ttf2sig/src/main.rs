use std::{
    convert::TryInto,
    fmt,
    io::BufWriter,
    num::{ParseFloatError, TryFromIntError},
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;
use color_eyre::eyre::{self, eyre, Context, ContextCompat};
use fontdue::VariationAxis;
use signum::{
    chsets::{
        editor::{EChar, ESet, ECHAR_NULL},
        metrics::FontMetrics,
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
/// Turn a TrueType/OpenType font file into a signum font
pub struct Opts {
    /// The file to convert
    font_file: PathBuf,

    /// The directory to output
    out: PathBuf,

    /// Generate an editor (.E24) file
    #[clap(short, long, default_value = "true")]
    editor: bool,

    #[clap(short, long, default_value = "ANTIKRO")]
    /// ToUnicode mapping name
    mapping: String,

    /// Force overwrite existing files
    #[clap(short, long)]
    force: bool,

    /// Assume the font size
    #[clap(short = 's', long, default_value = "10")]
    font_size: u32,

    /// Index (of a multi-font file)
    #[clap(short, long, default_value = "0")]
    index: u32,

    /// The new name of the font
    #[clap(short, long)]
    name: Option<String>,

    /// Threshold at which to treat coverage as "on"
    #[clap(short, long, default_value = "170")]
    threshold: u8,

    #[clap(short, long)]
    /// Specific OpenType font variations (AXIS=value)
    variation: Vec<Variation>,

    #[clap(long)]
    /// Variation: Italic (ital)
    italic: Option<f32>,

    #[clap(long)]
    /// Variation: Weight (wght)
    weight: Option<f32>,

    #[clap(long)]
    /// Variation: Width (wdth)
    width: Option<f32>,

    #[clap(long)]
    /// Variation: Optical Size (opsz)
    optical_size: Option<f32>,

    #[clap(long)]
    /// Variation: Grade Axis (GRAD)
    grade: Option<f32>,
}

#[derive(Debug, Copy, Clone)]
struct Variation {
    axis: VariationAxis,
    value: f32,
}

fn variation_tag(s: &str) -> Option<VariationAxis> {
    if s.is_ascii() && s.len() == 4 {
        let mut chars = s.chars();
        let a = chars.next()?;
        let b = chars.next()?;
        let c = chars.next()?;
        let d = chars.next()?;
        Some(VariationAxis::from_bytes([
            a as u8, b as u8, c as u8, d as u8,
        ]))
    } else {
        None
    }
}

#[derive(Debug, Clone)]
enum VariationError {
    NoEquals,
    MalformedTag,
    #[allow(dead_code)]
    Value(ParseFloatError),
}

impl From<ParseFloatError> for VariationError {
    fn from(value: ParseFloatError) -> Self {
        Self::Value(value)
    }
}

impl std::error::Error for VariationError {}

impl fmt::Display for VariationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

impl FromStr for Variation {
    type Err = VariationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (first, rest) = s.split_once("=").ok_or(VariationError::NoEquals)?;
        let axis = variation_tag(first).ok_or(VariationError::MalformedTag)?;
        let value = f32::from_str(rest)?;
        Ok(Variation { axis, value })
    }
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
    let mut font_settings = fontdue::FontSettings::default();
    font_settings.collection_index = opt.index;
    font_settings.load_substitutions = true;

    if let Some(grad) = opt.grade {
        font_settings
            .variations
            .insert(VariationAxis::from_bytes(*b"GRAD"), grad);
    }
    if let Some(ital) = opt.italic {
        font_settings.variations.insert(VariationAxis::ITALIC, ital);
    }
    if let Some(opsz) = opt.optical_size {
        font_settings
            .variations
            .insert(VariationAxis::OPTICAL_SIZE, opsz);
    }
    if let Some(wdth) = opt.width {
        font_settings.variations.insert(VariationAxis::WIDTH, wdth);
    }
    if let Some(wght) = opt.weight {
        font_settings.variations.insert(VariationAxis::WEIGHT, wght);
    }
    for var in &opt.variation {
        font_settings.variations.insert(var.axis, var.value);
    }
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
    let pk = PrinterKind::Needle24;
    let fm = FontMetrics::new(pk, opt.font_size);
    let px_per_em = fm.em_square_pixels();
    dbg!(px_per_em);

    let editor_font_metrics = FontMetrics::new(FontKind::Editor, opt.font_size);
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
        if opt.editor {
            eset_chars.push(make_echar(e_metrics, e_bitmap).unwrap());
        }
    }
    let out_dir = &opt.out;
    let pset = PSet {
        pk,
        header: Buf(&[0u8; 128]),
        chars: pset_chars,
    };
    write_pset(&name, pset, out_dir, opt.force)?;
    if opt.editor {
        let eset = ESet {
            buf1: Buf(&[0u8; 128]),
            chars: eset_chars,
        };
        write_eset(&name, eset, out_dir, opt.force)?;
    }

    Ok(())
}

fn find_glyph(
    face: &ttf_parser::Face<'_>,
    ligatures: &LigatureInfo<'_>,
    c: &[char],
) -> Option<GlyphId> {
    match c {
        [] | ['\0'] | [char::REPLACEMENT_CHARACTER] => None,
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
    let ymin_from_top = (FontKind::Editor.baseline() as i32 - metrics.ymin) as usize;
    let top = if ymin_from_top >= metrics.height {
        (ymin_from_top - metrics.height) as u8
    } else {
        // bitmap = bitmap.v_offset((metrics.height - ymin_from_top) as u32);
        0
    };
    let height = bitmap.bit_height().try_into()?;
    Ok(EChar::new_owned(width, height, top, bitmap.into_vec()).unwrap())
}

fn make_pchar(
    pk: PrinterKind,
    metrics: fontdue::Metrics,
    bitmap: signum::raster::Page,
) -> PSetChar<'static> {
    let ymin_from_top = (pk.baseline() as i32 - metrics.ymin) as usize;
    let top = if ymin_from_top >= metrics.height {
        (ymin_from_top - metrics.height) as u8
    } else {
        eprintln!(
            "WARN: glyph too high ({}px above {}, avail. ascent is {}), adjusting y!",
            metrics.height,
            metrics.ymin,
            pk.baseline()
        );
        // bitmap = bitmap.v_offset(bitmap.bit_height() - ymin_from_top as u32);
        0
    };
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
        if img.width() + lpad as u32 > max as u32 {
            if img.width() <= max as u32 {
                println!("WARN: editor font limited to {max} width, reducing left bearing");
                lpad = lpad.min(max as i32 - img.width() as i32);
            } else {
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
    }
    let covered = lpad as u8 + metrics.width as u8;
    let rpad = match req_width {
        Some(w) if w > covered => w - covered,
        _ => 0,
    };
    if let Some(max_bits) = req_width {
        let sum = img.width() + lpad as u32 + rpad as u32;
        assert!(
            sum <= max_bits.into(),
            "{} < {} (l+{},r+{})",
            img.width(),
            max_bits,
            lpad,
            rpad
        );
    }
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
