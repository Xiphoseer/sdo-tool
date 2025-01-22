//! # Signum! file tool
#![warn(missing_docs)]

use color_eyre::eyre::{self, eyre, WrapErr};
use log::{error, info, LevelFilter};
use sdo_tool::cli::{
    bimc::process_bimc,
    font::{process_eset, process_ls30, process_ps09, process_ps24},
    opt::Options,
    sdoc::process_sdoc,
};
use signum::{docs::four_cc, util::FourCC};
use std::{
    fs::File,
    io::{BufReader, Read},
};
use structopt::StructOpt;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .init();
    let opt = Options::from_args();
    let file_res = File::open(&opt.file);
    let file = WrapErr::wrap_err_with(file_res, || {
        format!("Failed to open file: `{}`", opt.file.display())
    })?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    info!("Loaded file `{}`", opt.file.display());

    let (_, four_cc) = four_cc::<signum::nom::error::Error<&'_ [u8]>>(&buffer[..])
        .map_err(|_| eyre!("File has less than 4 bytes"))?;
    match four_cc {
        FourCC::SDOC => process_sdoc(&buffer, opt),
        FourCC::ESET => process_eset(&buffer, None, None),
        FourCC::PS09 => process_ps09(&buffer, &opt),
        FourCC::PS24 => process_ps24(&buffer, &opt),
        FourCC::LS30 => process_ls30(&buffer, &opt),
        FourCC::BIMC => process_bimc(&buffer, opt),
        fourcc => {
            error!("Unknown file type {fourcc}");
            Ok(())
        }
    }
}
