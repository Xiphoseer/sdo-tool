use std::{borrow::Cow, collections::BTreeMap, fmt, io, path::PathBuf, str::FromStr};

use pdf_create::high;
use serde::Deserialize;
use signum::chsets::FontKind;
use structopt::StructOpt;
use thiserror::*;

mod de;
use de::{deserialize_opt_i32, deserialize_opt_string};

/// The format to export the document into
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum Format {
    /// Plain utf-8 text
    #[default]
    Plain,
    /// Text with formatting annotations (Documents)
    Html,
    /// PostScript page description file (Documents)
    PostScript,
    /// Portable Document Format (Documents)
    Pdf,
    /// A list of draw commands (Documents)
    PDraw,
    /// Protable Network Graphic (Documents, Images)
    Png,
    /// Portable Bitmap Format (Images)
    Pbm,

    /// A dvips-compatible inline postscript bitmap font (unstable)
    DviPsBitmapFont,
    /// A sequence of CCITT group 4 encoded bitmaps (Fonts)
    CcItt6,
}

#[derive(Debug)]
/// Failed to parse a format name
pub struct FormatError {}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Use one of `plain`, `html`, `pdf`, `ps`, `png`, `pbm` or `pdraw`"
        )?;
        Ok(())
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
            "pdf" => Ok(Self::Pdf),
            "pbm" => Ok(Self::Pbm),
            "pdraw" => Ok(Self::PDraw),
            "dvipsbf" => Ok(Self::DviPsBitmapFont),
            "ccitt-t6" => Ok(Self::CcItt6),
            _ => Err(FormatError {}),
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain => f.write_str("plain"),
            Self::Html => f.write_str("html"),
            Self::PostScript => f.write_str("ps"),
            Self::Png => f.write_str("png"),
            Self::Pbm => f.write_str("pbm"),
            Self::Pdf => f.write_str("pdf"),
            Self::PDraw => f.write_str("pdraw"),
            Self::DviPsBitmapFont => f.write_str("dvipsbf"),
            Self::CcItt6 => f.write_str("ccitt-t6"),
        }
    }
}
#[derive(StructOpt)]
/// Convert a Signum file to another format
pub struct Options {
    /// The file to be processed (e.g. *.SDO, *.E24, *.IMC)
    pub file: PathBuf,
    /// Where to store the output
    pub out: Option<PathBuf>,
    /// If specified, extract all embedded images to that folder
    #[structopt(long = "with-images", short = "I")]
    pub with_images: Option<PathBuf>,
    /// Select the printer font (and resolution).
    ///
    /// May fail, if the fonts are not available.
    #[structopt(long = "print-driver", short = "P")]
    pub print_driver: Option<FontKind>,
    #[structopt(long, short = "C", default_value = "CHSETS")]
    pub chsets_path: PathBuf,
    /// If specified, limits the pages that are printed
    #[structopt(long = "page", short = "#")]
    pub page: Option<Vec<usize>>,
    /// Format of the output. Valid choices are:
    ///
    /// "plain", "html", "pdf", "ps", "png", "pbm" and "pdraw"
    #[structopt(default_value, long, short = "F")]
    pub format: Format,

    /// Meta Parameters passed as command line args
    #[structopt(flatten)]
    pub cl_meta: Meta,

    /// Meta parameter as a file
    #[structopt(long)]
    pub meta: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum MetaError {
    #[error("IO Error")]
    Io(#[from] io::Error),
    #[error("Deserialize Error")]
    Ron(#[from] ron::error::Error),
}

impl Options {
    pub fn meta(&self) -> Result<Cow<Meta>, MetaError> {
        if let Some(meta_path) = &self.meta {
            let text = std::fs::read_to_string(meta_path)?;
            let mut meta: Meta = ron::from_str(&text)?;
            if let Some(xoffset) = self.cl_meta.xoffset {
                meta.xoffset = Some(xoffset);
            }
            if let Some(yoffset) = self.cl_meta.yoffset {
                meta.yoffset = Some(yoffset);
            }
            if let Some(author) = &self.cl_meta.author {
                meta.author = Some(author.clone());
            }
            if let Some(title) = &self.cl_meta.title {
                meta.title = Some(title.clone());
            }
            if let Some(subject) = &self.cl_meta.subject {
                meta.subject = Some(subject.clone());
            }
            Ok(Cow::Owned(meta))
        } else {
            Ok(Cow::Borrowed(&self.cl_meta))
        }
    }
}

#[derive(Debug, Default, Clone, StructOpt, Deserialize)]
pub struct Meta {
    /// Horizontal offset
    #[structopt(long)]
    #[serde(default, deserialize_with = "deserialize_opt_i32")]
    pub xoffset: Option<i32>,
    /// Vertical offset
    #[structopt(long)]
    #[serde(default, deserialize_with = "deserialize_opt_i32")]
    pub yoffset: Option<i32>,
    /// Author
    #[structopt(long)]
    #[serde(default, deserialize_with = "deserialize_opt_string")]
    pub author: Option<String>,
    /// Title
    #[structopt(long)]
    #[serde(default, deserialize_with = "deserialize_opt_string")]
    pub title: Option<String>,
    /// Subject
    #[structopt(long)]
    #[serde(default, deserialize_with = "deserialize_opt_string")]
    pub subject: Option<String>,
}

fn chsets_path() -> PathBuf {
    PathBuf::from("CHSETS")
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct DocScript {
    /// The document meta information
    #[serde(default)]
    pub meta: Meta,

    /// The files the constitute the document
    pub files: Vec<PathBuf>,

    /// The page labels
    #[serde(default)]
    pub page_labels: BTreeMap<usize, PageLabel>,

    /// The root outline items
    #[serde(default)]
    pub outline: Vec<OutlineItem>,

    /// The path to the fonts folder
    #[serde(default = "chsets_path")]
    pub chsets: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutlineItem {
    /// The title of the outline item
    pub title: String,
    /// The destination to navigate to
    pub dest: Destination,
    /// Immediate children of this item
    #[serde(default)]
    pub children: Vec<OutlineItem>,
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub enum Destination {
    PageFitH(usize, usize),
}

impl From<Destination> for high::Destination {
    fn from(d: Destination) -> Self {
        match d {
            Destination::PageFitH(a, b) => Self::PageFitH(a, b),
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub enum PageLabelKind {
    None,
    Decimal,
    RomanLower,
    RomanUpper,
    AlphaLower,
    AlphaUpper,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageLabel {
    pub prefix: String,
    pub kind: PageLabelKind,
    pub start: u32,
}

impl From<PageLabelKind> for Option<pdf_create::common::PageLabelKind> {
    fn from(kind: PageLabelKind) -> Self {
        use pdf_create::common::PageLabelKind::*;
        match kind {
            PageLabelKind::None => None,
            PageLabelKind::Decimal => Some(Decimal),
            PageLabelKind::RomanLower => Some(RomanLower),
            PageLabelKind::RomanUpper => Some(RomanUpper),
            PageLabelKind::AlphaLower => Some(AlphaLower),
            PageLabelKind::AlphaUpper => Some(AlphaUpper),
        }
    }
}
