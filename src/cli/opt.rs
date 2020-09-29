use std::{fmt, path::PathBuf, str::FromStr};

use sdo::font::printer::PrintDriver;
use structopt::StructOpt;

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

    /// A dvips-compatible inline postscript bitmap font (unstable)
    DVIPSBitmapFont,
    /// A sequence of CCITT group 4 encoded bitmaps (unstable)
    CCITTT6,
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
            "dvipsbf" => Ok(Self::DVIPSBitmapFont),
            "ccitt-t6" => Ok(Self::CCITTT6),
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
            Self::DVIPSBitmapFont => f.write_str("dvipsbf"),
            Self::CCITTT6 => f.write_str("ccitt-t6"),
        }
    }
}

/// OPTIONS
#[derive(StructOpt)]
pub struct Options {
    /// Where to store the output
    pub out: PathBuf,
    /// If specified, extract all embedded images to that folder
    #[structopt(long = "with-images", short = "I")]
    pub with_images: Option<PathBuf>,
    /// Select the printer font (and resolution).
    ///
    /// May fail, if the fonts are not available.
    #[structopt(long = "print-driver", short = "P")]
    pub print_driver: Option<PrintDriver>,
    /// If specified, limits the pages that are printed
    #[structopt(long = "page", short = "#")]
    pub page: Option<Vec<usize>>,
    /// Format of the output. Valid choices are:
    ///
    /// "txt", "html", "ps", "png", and "pdraw"
    #[structopt(default_value, long, short = "F")]
    pub format: Format,

    /// HACK: fix horizontal offset
    #[structopt(long)]
    pub xoffset: Option<isize>,
}
