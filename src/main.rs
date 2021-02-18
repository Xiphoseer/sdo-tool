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
};
use structopt::StructOpt;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let opt = Options::from_args();

    let file_res = File::open(&opt.file);
    let file = WrapErr::wrap_err_with(file_res, || {
        format!("Failed to open file: `{}`", opt.file.display())
    })?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    match buffer.get(..4) {
        Some(b"sdoc") => process_sdoc(&buffer, opt),
        Some(b"eset") => process_eset(&buffer, None, None),
        Some(b"ps09") => process_ps09(&buffer, &opt),
        Some(b"ps24") => process_ps24(&buffer, &opt),
        Some(b"ls30") => process_ls30(&buffer, &opt),
        Some(b"bimc") => process_bimc(&buffer, opt),
        Some(t) => Err(eyre!("Unknown file type {:?}", t)),
        None => Err(eyre!("File has less than 4 bytes")),
    }
}
