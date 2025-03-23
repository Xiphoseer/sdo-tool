use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::{self, eyre};
use signum::util::{FileFormatKind, SignumFormat};

#[derive(clap::Parser)]
/// Describe the type of the Signum file
struct Options {
    /// A signum file
    file: PathBuf,
}

fn info(buffer: &[u8], opt: &Options) -> color_eyre::Result<()> {
    if let Some(ff) = SignumFormat::detect(buffer) {
        println!("{}", ff.file_format_name());
        println!("Run `sdo-tool {:?}` to learn more", opt.file);
        Ok(())
    } else {
        Err(eyre!("Unknown file type"))
    }
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let opt: Options = Options::parse();

    let buffer = std::fs::read(&opt.file)?;
    info(&buffer, &opt)
}
