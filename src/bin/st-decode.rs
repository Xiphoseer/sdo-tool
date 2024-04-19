use std::path::PathBuf;

use color_eyre::eyre;
use structopt::StructOpt;

#[derive(StructOpt)]
/// Options for decoding an ATARI String
pub struct DecodeOpts {
    /// The file to convert
    file: PathBuf,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let opt: DecodeOpts = DecodeOpts::from_args();

    let buffer = std::fs::read(opt.file)?;

    let mut decoded = String::with_capacity(buffer.len());
    for byte in buffer {
        let ch = signum::chsets::encoding::decode_atari(byte);
        decoded.push(ch);
    }
    print!("{}", decoded);
    Ok(())
}
