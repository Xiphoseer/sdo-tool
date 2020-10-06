use std::path::PathBuf;

use color_eyre::eyre::{self, WrapErr};
use structopt::StructOpt;

use super::opt::DocScript;

#[derive(StructOpt, Debug)]
pub struct RunOpts {
    out: PathBuf,
}

pub fn run(_file: PathBuf, buffer: &[u8], opt: RunOpts) -> eyre::Result<()> {
    let script_str_res = std::str::from_utf8(buffer);
    let script_str = WrapErr::wrap_err(script_str_res, "Failed to parse as string")?;
    let script_res = ron::from_str(script_str);
    let script: DocScript = WrapErr::wrap_err(script_res, "Failed to parse DocScript")?;
    println!("script: {:#?}", script);
    println!("opt: {:?}", opt);
    Ok(())
}
