use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::{self, eyre};

#[derive(clap::Parser)]
/// Describe the type of the Signum file
struct Options {
    /// A signum file
    file: PathBuf,
}

fn info(buffer: &[u8], opt: Options) -> color_eyre::Result<()> {
    match buffer.get(..4) {
        Some(b"sdoc") => {
            println!("Signum!2 Document");
            println!("Use `sdo-tool \"{}\"` to learn more", opt.file.display());
            Ok(())
        }
        Some(b"eset") => {
            println!("Signum!2 Editor Font");
            println!("Use `sdo-tool {}` to learn more", opt.file.display());
            Ok(())
        }
        Some(b"bimc") => {
            println!("Signum!2 Compressed Image");
            println!("Use `sdo-tool {}` to learn more", opt.file.display());
            Ok(())
        }
        Some(b"ls30") => {
            println!("Signum!2 30-Point Laser Printer Font");
            println!("Use `sdo-tool {}` to learn more", opt.file.display());
            Ok(())
        }
        Some(b"ps24") => {
            println!("Signum!2 24-Needle Printer Font");
            println!("Use `sdo-tool {}` to learn more", opt.file.display());
            Ok(())
        }
        Some(b"ps09") => {
            println!("Signum!2 9-Needle Printer Font");
            println!("Use `sdo-tool {}` to learn more", opt.file.display());
            Ok(())
        }
        Some(b"cryp") => {
            println!("Papyrus Encrypted Font (?)");
            println!("Currently not supported!");
            Ok(())
        }
        Some(t) => Err(eyre!("Unknown file type {:?}", t)),
        None => Err(eyre!("File has less than 4 bytes")),
    }
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let opt: Options = Options::parse();

    let buffer = std::fs::read(&opt.file)?;
    info(&buffer, opt)
}
