use std::{borrow::Cow, collections::BTreeMap, fmt, io, path::PathBuf, str::FromStr};

use clap::Parser;
use pdf_create::high;
use sdo_pdf::MetaInfo;
use serde::{Deserialize, Serialize};
use signum::{chsets::FontKind, docs::Overrides};
use thiserror::*;

mod de;
use de::{deserialize_opt_i32, deserialize_opt_string, deserialize_string_or_list};

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
    /// Glyph Bitmap Distribution Format (Fonts)
    Bdf,
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

impl std::error::Error for FormatError {}

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
            "ccitt" | "ccitt-t6" => Ok(Self::CcItt6),
            _ => Err(FormatError {}),
        }
    }
}

impl Format {
    fn to_static_str(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Html => "html",
            Self::PostScript => "ps",
            Self::Png => "png",
            Self::Pbm => "pbm",
            Self::Pdf => "pdf",
            Self::PDraw => "pdraw",
            Self::DviPsBitmapFont => "dvipsbf",
            Self::CcItt6 => "ccitt-t6",
            Self::Bdf => "bdf",
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_static_str())
    }
}
#[derive(Parser)]
/// Convert a Signum file to another format
pub struct Options {
    /// The file to be processed (e.g. *.SDO, *.E24, *.IMC)
    pub file: PathBuf,
    /// Where to store the output
    pub out: Option<PathBuf>,
    /// If specified, extract all embedded images to that folder
    #[clap(long = "with-images", short = 'I')]
    pub with_images: Option<PathBuf>,
    /// Select the printer font (and resolution).
    ///
    /// May fail, if the fonts are not available.
    #[clap(long = "print-driver", short = 'P')]
    pub print_driver: Option<FontKind>,
    #[clap(long, short = 'C', default_value = "CHSETS")]
    pub chsets_path: PathBuf,
    /// If specified, limits the pages that are printed
    #[clap(long = "page", short = '#')]
    pub page: Option<Vec<usize>>,
    /// Format of the output. Valid choices are:
    ///
    /// "plain", "html", "pdf", "ps", "png", "pbm", "bdf" and "pdraw"
    #[clap(default_value_t, long, short = 'F')]
    pub format: Format,

    /// Meta Parameters passed as command line args
    #[clap(flatten)]
    pub cl_meta: Meta,

    /// Meta parameter as a file
    #[clap(long)]
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
            meta.author = self.cl_meta.author.clone();
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

#[derive(Debug, Default, Clone, Parser, Deserialize)]
pub struct Meta {
    /// Horizontal offset
    #[clap(long)]
    #[serde(default, deserialize_with = "deserialize_opt_i32")]
    pub xoffset: Option<i32>,
    /// Vertical offset
    #[clap(long)]
    #[serde(default, deserialize_with = "deserialize_opt_i32")]
    pub yoffset: Option<i32>,
    /// Author
    #[clap(long)]
    #[serde(default, deserialize_with = "deserialize_string_or_list")]
    pub author: Vec<String>,
    /// Title
    #[clap(long)]
    #[serde(default, deserialize_with = "deserialize_opt_string")]
    pub title: Option<String>,
    /// Subject
    #[clap(long)]
    #[serde(default, deserialize_with = "deserialize_opt_string")]
    pub subject: Option<String>,
}

impl Meta {
    /// Get the [MetaInfo] for PDF generation
    pub fn pdf_meta_info(&self, file_name: &str) -> MetaInfo {
        MetaInfo {
            author: self.author.to_owned(),
            title: Some(
                match self.title.as_deref() {
                    Some(title) => title,
                    None => file_name,
                }
                .to_owned(),
            ),
            subject: self.subject.to_owned(),
            ..Default::default()
        }
    }

    /// Get the [MetaInfo] for PDF generation
    pub fn to_pdf_meta(&self) -> MetaInfo {
        MetaInfo {
            author: self.author.to_owned(),
            title: self.title.to_owned(),
            subject: self.subject.to_owned(),
            ..Default::default()
        }
    }

    pub fn to_overrides(&self) -> Overrides {
        Overrides {
            xoffset: self.xoffset.unwrap_or(0),
            yoffset: self.yoffset.unwrap_or(0),
        }
    }
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

    #[serde(default)]
    pub outline_file: Option<PathBuf>,

    /// The path to the fonts folder
    #[serde(default = "chsets_path")]
    pub chsets: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutlineItem {
    /// The title of the outline item
    pub title: String,
    /// The destination to navigate to
    pub dest: Destination,
    /// Immediate children of this item
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<OutlineItem>,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
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
