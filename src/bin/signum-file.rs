use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::{self, eyre};
use signum::{docs::four_cc, nom::error::Error};

#[derive(clap::Parser)]
/// Describe the type of the Signum file
struct Options {
    /// A signum file
    file: PathBuf,
}

fn info(buffer: &[u8], opt: &Options) -> color_eyre::Result<()> {
    let (_, four_cc) =
        four_cc::<Error<_>>(buffer).map_err(|_e| eyre!("File has less than 4 bytes"))?;
    if let Some(ff) = four_cc.file_format_name() {
        println!("{}", ff);
        println!("Run `sdo-tool {:?}` to learn more", opt.file);
        Ok(())
    } else if buffer.starts_with(b"\0\0sdoc  03\0\0") {
        println!("Signum! 3/4 document");
        Ok(())
    } else {
        Err(eyre!("Unknown file type {:?}", four_cc))
    }
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let opt: Options = Options::parse();

    let buffer = std::fs::read(&opt.file)?;
    info(&buffer, &opt)
}
