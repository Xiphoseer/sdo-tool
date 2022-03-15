#![allow(unused)]
use clap::Parser;
use color_eyre::eyre::{self, eyre, WrapErr};
use image::ImageFormat;
use sdo_util::keymap::{print_eset, Draw, KB_DRAW, NP_DRAW};
use signum::{
    chsets::editor::{parse_eset, EChar, ESet, OwnedESet, ECHAR_NULL},
    raster::Page,
};
use std::{
    collections::HashSet,
    ffi::{OsStr, OsString},
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

#[derive(Parser)]
/// Print a keyboard for the given font
pub struct KbOptions {
    /// An .E24 file
    file: PathBuf,
    /// The output image
    out: PathBuf,
}

pub const CHARS_GRAPH: [u8; 9] = [49, 64, 101, 112, 113, 114, 116, 119, 122];
pub const CHARS_LABEL: [u8; 25] = [
    65, 66, 67, 68, 69, 76, 82, 83, 84, 97, 98, 99, 101, 102, 104, 105, 107, 108, 110, 111, 112,
    114, 115, 116, 117,
];

fn _run_stage_2() -> eyre::Result<()> {
    let gfont = OwnedESet::load(Path::new("../chsets/GRAPH1.E24"))?;
    let lfont = OwnedESet::load(Path::new("../chsets/GROTMIKR.E24"))?;

    print_eset(&CHARS_GRAPH[..], &gfont, "gchar");
    print_eset(&CHARS_LABEL[..], &lfont, "lchar");
    Ok(())
}

fn make(draw: &Draw, eset: &ESet, out: &Path, file_name: &OsStr, key: &str) -> eyre::Result<()> {
    let img = draw.to_page(eset)?.to_image();

    // Prepare file name
    let mut name = OsString::from(key);
    name.push(file_name);
    let mut path = out.join(name);
    path.set_extension("png");

    img.save_with_format(&path, ImageFormat::Png)?;

    Ok(())
}

pub fn run(buffer: &[u8], opt: KbOptions) -> eyre::Result<()> {
    let (_, eset) = parse_eset(buffer) //
        .map_err(|e| eyre!("Could not load editor charset:\n{}", e))?;

    // Create dir if not exists
    if !opt.out.exists() {
        std::fs::create_dir(&opt.out);
    }

    let file_name = opt.file.file_name().unwrap();
    make(&KB_DRAW, &eset, &opt.out, file_name, "kb")?; // Keyboard
    make(&NP_DRAW, &eset, &opt.out, file_name, "np")?; // Numpad

    Ok(())
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let opt: KbOptions = KbOptions::parse();

    let file_res = File::open(&opt.file);
    let file = WrapErr::wrap_err_with(file_res, || {
        format!("Failed to open file: `{}`", opt.file.display())
    })?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    run(&buffer, opt)
}
