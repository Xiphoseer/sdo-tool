//! # Signum! file tool
#![warn(missing_docs)]

mod cli;
mod font;
mod images;
mod print;
mod ps;
mod sdoc;
mod util;

use crate::util::Buf;

use anyhow::anyhow;
use cli::{
    keyboard, process_bimc, process_eset, process_ls30, process_ps24,
    ps::convert_ls30,
    ps::process_ps_font,
    sdoc::{process_sdoc, PrintDriver},
};
use keyboard::KBOptions;
use std::{
    fmt,
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
    str::FromStr,
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

/// The format to export the document into
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Format {
    /// Plain utf-8 text
    Plain,
    /// Text with formatting annotations
    Html,
    /// PostScript page description file
    PostScript,
    /// A Sequence of images
    Png,
    /// A list of draw commands
    PDraw,
}

#[derive(Debug)]
/// Failed to parse a format name
pub struct FormatError {}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Use one of `plain`, `html`, `ps`, `png` or `pdraw`")?;
        Ok(())
    }
}

impl Default for Format {
    fn default() -> Self {
        Format::Html
    }
}

impl FromStr for Format {
    type Err = FormatError;
    fn from_str(val: &str) -> Result<Self, Self::Err> {
        match val {
            "txt" | "plain" => Ok(Self::Plain),
            "html" => Ok(Self::Html),
            "ps" | "postscript" => Ok(Self::PostScript),
            "png" => Ok(Self::Png),
            "pdraw" => Ok(Self::PDraw),
            _ => Err(FormatError {}),
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain => f.write_str("txt"),
            Self::Html => f.write_str("html"),
            Self::PostScript => f.write_str("ps"),
            Self::Png => f.write_str("png"),
            Self::PDraw => f.write_str("pdraw"),
        }
    }
}

/// OPTIONS
#[derive(StructOpt)]
pub struct Options {
    /// Where to store the output
    out: PathBuf,
    /// Where to store the image output, if applicable
    #[structopt(long = "with-images")]
    with_images: Option<PathBuf>,
    #[structopt(long = "print-driver", short = "P")]
    print_driver: Option<PrintDriver>,
    #[structopt(long = "pages", short = "#")]
    page: Option<Vec<usize>>,
    #[structopt(default_value, long)]
    format: Format,
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
            Some(b"/Fa ") => {
                process_ps_font(&buffer)?;
                Ok(())
            }
            Some(b"ls30") => {
                convert_ls30(&buffer)?;
                Ok(())
            }
            Some(t) => Err(anyhow!("Unknown file type {:?}", t)),
            None => Err(anyhow!("File has less than 4 bytes")),
        },
        Some(Command::Dump(dump_opt)) => match buffer.get(..4) {
            Some(b"sdoc") => process_sdoc(&buffer, dump_opt, &opt.file),
            Some(b"eset") => process_eset(&buffer, None, None),
            Some(b"ps24") => process_ps24(&buffer, &dump_opt),
            Some(b"ls30") => process_ls30(&buffer, &dump_opt),
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
        Some(Command::Keyboard(kbopt)) => keyboard::run(&opt.file, &buffer, kbopt),
    }
}
