use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::eyre;
use signum::images::imc::parse_imc;

#[derive(Debug, Parser)]
/// Converts a file from Signum! IMC to a Portable Bit-Map
struct Opts {
    /// The file to convert
    file: PathBuf,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let opts: Opts = Opts::parse();

    let file = std::fs::read(opts.file)?;
    let (_, image) = parse_imc(&file).map_err(|e| eyre!("{}", e))?;
    println!("P1 640 400");
    for line in image.into_inner().chunks(8) {
        for byte in line {
            print!("{:08b}", byte);
        }
        println!();
    }

    Ok(())
}
