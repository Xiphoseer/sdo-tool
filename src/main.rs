//! # Signum! file tool
#![warn(missing_docs)]

use color_eyre::eyre::{self, eyre, WrapErr};
use sdo_tool::cli::{
    font::{process_eset, process_ls30, process_ps09, process_ps24},
    opt::Options,
    process_bimc,
    sdoc::process_sdoc,
};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};
use structopt::StructOpt;

#[derive(StructOpt)]
/// The options for this CLI
pub struct CLI {
    /// The file to be processed
    file: PathBuf,

    /// How to process that file
    #[structopt(flatten)]
    dump_opt: Options,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let opt = CLI::from_args();

    let file_res = File::open(&opt.file);
    let file = WrapErr::wrap_err_with(file_res, || {
        format!("Failed to open file: `{}`", opt.file.display())
    })?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    match buffer.get(..4) {
        Some(b"sdoc") => process_sdoc(&buffer, opt.dump_opt, &opt.file),
        Some(b"eset") => process_eset(&buffer, None, None),
        Some(b"ps09") => process_ps09(&buffer, &opt.dump_opt),
        Some(b"ps24") => process_ps24(&buffer, &opt.dump_opt),
        Some(b"ls30") => process_ls30(&buffer, &opt.file, &opt.dump_opt),
        Some(b"bimc") => process_bimc(&buffer, &opt.file, opt.dump_opt.out),
        Some(t) => Err(eyre!("Unknown file type {:?}", t)),
        None => Err(eyre!("File has less than 4 bytes")),
    }
}
