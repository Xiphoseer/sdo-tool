//! # Signum! file tool
#![warn(missing_docs)]

use color_eyre::eyre::{self, WrapErr};
use log::{error, info, LevelFilter};
use sdo_tool::cli::{
    bimc::process_bimc,
    font::{process_eset, process_ls30, process_ps09, process_ps24},
    opt::Options,
    sdoc::process_sdoc,
};
use std::{
    fmt,
    fs::File,
    io::{BufReader, Read},
};
use structopt::StructOpt;

struct FourCc([u8; 4]);

impl fmt::Display for FourCc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &b in &self.0 {
            match b {
                b'\\' => write!(f, "\\\\"),
                b'"' => write!(f, "\\\""),
                32..=33 | 35..=91 | 93..=127 => write!(f, "{}", b as char),
                _ => write!(f, "\\x{:02x}", b),
            }?;
        }
        Ok(())
    }
}

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

    match buffer.get(..4) {
        Some(b"sdoc") => process_sdoc(&buffer, opt),
        Some(b"eset") => process_eset(&buffer, None, None),
        Some(b"ps09") => process_ps09(&buffer, &opt),
        Some(b"ps24") => process_ps24(&buffer, &opt),
        Some(b"ls30") => process_ls30(&buffer, &opt),
        Some(b"bimc") => process_bimc(&buffer, opt),
        Some(t) => {
            let fourcc = FourCc([t[0], t[1], t[2], t[3]]);
            error!("Unknown file type b\"{}\"", fourcc);
            Ok(())
        }
        None => {
            error!("File has less than 4 bytes");
            Ok(())
        }
    }
}
