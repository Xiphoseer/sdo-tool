//! # Signum! file tool
#![warn(missing_docs)]

mod cli;

use cli::opt::Options;
use sdo::font;

use cli::{
    font::ps::{convert_ls30, process_ps_font},
    keyboard, process_bimc, process_eset, process_ls30, process_ps24,
    sdoc::process_sdoc,
};
use color_eyre::eyre::{self, eyre};
use keyboard::KBOptions;
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
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

/// Subcommands
#[derive(StructOpt)]
pub enum Command {
    /// Dump the content of this file
    Dump(Options),
    /// Options for decoding an ATARI String
    Decode,
    /// Print a keyboard for the given font
    Keyboard(KBOptions),
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let opt = CLI::from_args();

    let file = File::open(&opt.file)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    match opt.cmd {
        None => match buffer.get(..4) {
            Some(b"sdoc") => {
                println!("Signum!2 Document");
                let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("sdo-tool"));
                let name = exe.file_name().unwrap().to_string_lossy();
                println!(
                    "Use `{} \"{}\" dump` to learn more",
                    name,
                    opt.file.display()
                );
                Ok(())
            }
            Some(b"eset") => {
                println!("Signum!2 Editor Font");
                println!("Use `sdo-tool {} dump` to learn more", opt.file.display());
                Ok(())
            }
            Some(b"bimc") => {
                println!("Signum!2 Compressed Image");
                println!("Use `sdo-tool {} dump` to learn more", opt.file.display());
                Ok(())
            }
            Some(b"/Fa ") => {
                process_ps_font(&buffer)?;
                Ok(())
            }
            Some(b"ls30") => {
                convert_ls30(&buffer)?;
                Ok(())
            }
            Some(t) => Err(eyre!("Unknown file type {:?}", t)),
            None => Err(eyre!("File has less than 4 bytes")),
        },
        Some(Command::Dump(dump_opt)) => match buffer.get(..4) {
            Some(b"sdoc") => process_sdoc(&buffer, dump_opt, &opt.file),
            Some(b"eset") => process_eset(&buffer, None, None),
            Some(b"ps24") => process_ps24(&buffer, &dump_opt),
            Some(b"ls30") => process_ls30(&buffer, &dump_opt),
            Some(b"bimc") => process_bimc(&buffer, dump_opt.out),
            Some(t) => Err(eyre!("Unknown file type {:?}", t)),
            None => Err(eyre!("File has less than 4 bytes")),
        },
        Some(Command::Decode) => {
            let mut decoded = String::with_capacity(buffer.len());
            for byte in buffer {
                let ch = font::decode_atari(byte);
                decoded.push(ch);
            }
            print!("{}", decoded);
            Ok(())
        }
        Some(Command::Keyboard(kbopt)) => keyboard::run(&opt.file, &buffer, kbopt),
    }
}
