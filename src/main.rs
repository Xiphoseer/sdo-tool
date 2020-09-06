//! # Signum! file tool
#![warn(missing_docs)]

mod cli;
mod font;
mod images;
mod print;
mod sdoc;
mod util;

use crate::{font::eset::parse_eset, images::imc::parse_imc, print::Page, util::Buf};

use anyhow::anyhow;
use cli::{keyboard, sdoc::process_sdoc};
use image::ImageFormat;
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

/// OPTIONS
#[derive(StructOpt)]
pub struct Options {
    /// Where to store the output, if applicable
    #[structopt(long)]
    out: Option<PathBuf>,
    /// Where to store the image output, if applicable
    #[structopt(long)]
    imout: Option<PathBuf>,
    /// Some input to process
    #[structopt(long)]
    input: Option<String>,
    #[structopt(long)]
    pdraw: bool,
}

fn process_eset(buffer: &[u8], input: Option<String>, out: Option<PathBuf>) -> anyhow::Result<()> {
    match parse_eset(buffer) {
        Ok((_rest, eset)) => {
            assert!(_rest.is_empty());
            if let Some(_a) = input {
                let mut page = Page::new(100, 24);

                let mut x = 0;
                for ci in /*48..58*/ (65..66).chain(98..109) {
                    let ch = &eset.chars[ci as usize];
                    page.draw_char(x, 0, ch).unwrap();
                    x += u16::from(ch.width) + 1;
                }

                if let Some(out_path) = out {
                    let image = page.to_image();
                    image.save_with_format(out_path, ImageFormat::Png)?;
                } else {
                    page.print();
                }
            } else {
                println!("{:#?}", eset.buf1);
                eset.print();
            }
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
    Ok(())
}

fn process_bimc(buffer: &[u8], out: Option<PathBuf>) -> anyhow::Result<()> {
    let decoded = parse_imc(&buffer) //
        .map_err(|err| anyhow!("Failed to parse: {}", err))?;

    let page = Page::from_screen(decoded);

    if let Some(out_path) = out {
        let image = page.to_image();
        image.save_with_format(out_path, ImageFormat::Png)?;
    } else {
        println!("Decoded image sucessfully, to store it as PNG, pass `--out <PATH>`");
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
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
            Some(t) => Err(anyhow!("Unknown file type {:?}", t)),
            None => Err(anyhow!("File has less than 4 bytes")),
        },
        Some(Command::Dump(dump_opt)) => match buffer.get(..4) {
            Some(b"sdoc") => process_sdoc(&buffer, dump_opt, &opt.file),
            Some(b"eset") => process_eset(&buffer, dump_opt.input, dump_opt.out),
            Some(b"bimc") => process_bimc(&buffer, dump_opt.out),
            Some(t) => Err(anyhow!("Unknown file type {:?}", t)),
            None => Err(anyhow!("File has less than 4 bytes")),
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
        Some(Command::Keyboard(kbopt)) => keyboard::run(&buffer, kbopt),
    }
}
