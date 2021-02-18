use std::path::PathBuf;

use color_eyre::eyre;
use sdo_tool::cli::font::ps::process_ps_font;
use structopt::StructOpt;
#[derive(StructOpt)]
/// Print information about a DVIPS Bitmap font
struct Options {
    /// Text of a DVIPSBitmapFont
    file: PathBuf,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let opt: Options = Options::from_args();

    let buffer = std::fs::read(&opt.file)?;
    process_ps_font(&buffer)
}
