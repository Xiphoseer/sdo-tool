use std::path::PathBuf;

use sdo_pdf::cmap::write_cmap;
use signum::chsets::encoding::p_mapping_file;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opts {
    file: PathBuf,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let opts: Opts = Opts::from_args();
    let input = std::fs::read_to_string(&opts.file)?;
    let stem = opts.file.file_stem().unwrap();
    let mapping = p_mapping_file(&input)?;

    let name = stem.to_string_lossy();
    let mut out = String::new();
    write_cmap(&mut out, &mapping, name.as_ref())?;
    print!("{}", out);

    Ok(())
}
