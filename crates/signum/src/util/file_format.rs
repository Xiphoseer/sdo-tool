use crate::{
    chsets::{v2::TAG_CSET2, FontKind},
    docs::v3::TAG_SDOC3,
};

use super::FourCC;

/// Trait for values that encode a kind of file format, e.g. [`crate::chsets::FontKind`]
pub trait FileFormatKind {
    /// Get the extension used for files of this type
    fn extension(&self) -> &'static str;

    /// Get the file format name for this printer kind
    fn file_format_name(&self) -> &'static str;
}

/// Trait for Signum!1/2 formats, which use simple FourCC codes
pub trait FileFormatKindV1: FileFormatKind {
    /// Get the magic used to detect files of this type
    fn magic(&self) -> FourCC;
}

/// Known Signum!1/2 formats
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Signum1Format {
    /// Document (sdoc)
    Document,
    /// Fonts (est, ps24, ps09, ls30)
    Font(FontKind),
    /// Hardcopy Image (bimc)
    HardcopyImage,
    /// Clipboard (sclb)
    Clipboard,
}

impl Signum1Format {
    /// Detect a version 1/2 format
    pub fn detect(data: &[u8]) -> Option<Self> {
        data.get(..4)
            .and_then(|b| FourCC::new([b[0], b[1], b[2], b[3]]).file_format())
    }
}

impl FileFormatKind for Signum1Format {
    fn extension(&self) -> &'static str {
        match self {
            Signum1Format::Document => "SDO",
            Signum1Format::Font(font_kind) => font_kind.extension(),
            Signum1Format::HardcopyImage => "IMC",
            Signum1Format::Clipboard => "CLB",
        }
    }

    fn file_format_name(&self) -> &'static str {
        match self {
            Signum1Format::Document => "Signum! Document",
            Signum1Format::Font(font_kind) => font_kind.file_format_name(),
            Signum1Format::HardcopyImage => "Signum! Hardcopy Image",
            Signum1Format::Clipboard => "Signum! Clipboard",
        }
    }
}

impl FileFormatKindV1 for Signum1Format {
    fn magic(&self) -> FourCC {
        match self {
            Signum1Format::Document => FourCC::SDOC,
            Signum1Format::Font(font_kind) => font_kind.magic(),
            Signum1Format::HardcopyImage => FourCC::BIMC,
            Signum1Format::Clipboard => FourCC::SCLB,
        }
    }
}

/// Signum 3/4 Format
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Signum3Format {
    /// Signum Document
    Document,
    /// Signum Font
    Font,
}

impl Signum3Format {
    /// Detect a Signum 3/4 format
    pub fn detect(data: &[u8]) -> Option<Self> {
        const SDOC3: &[u8] = TAG_SDOC3;
        const CSET2: &[u8] = TAG_CSET2;
        match data.get(..12)? {
            SDOC3 => Some(Self::Document),
            CSET2 => Some(Self::Font),
            _ => None,
        }
    }
}

impl FileFormatKind for Signum3Format {
    fn extension(&self) -> &'static str {
        match self {
            Signum3Format::Document => "SDK",
            Signum3Format::Font => "S01", // 9P, 24P, 30L
        }
    }

    fn file_format_name(&self) -> &'static str {
        match self {
            Signum3Format::Document => "Signum! 3/4 Document",
            Signum3Format::Font => "Signum! 3/4 Font",
        }
    }
}

/// Overall signum format
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SignumFormat {
    /// Signum 1/2 format (using FourCC)
    Signum1(Signum1Format),
    /// Signum 3/4 format (using 12-byte tag)
    Signum3(Signum3Format),
}

impl SignumFormat {
    /// Detect a signum format
    pub fn detect(data: &[u8]) -> Option<Self> {
        Signum1Format::detect(data)
            .map(SignumFormat::Signum1)
            .or_else(|| Signum3Format::detect(data).map(SignumFormat::Signum3))
    }
}

impl FileFormatKind for SignumFormat {
    fn extension(&self) -> &'static str {
        match self {
            SignumFormat::Signum1(f) => f.extension(),
            SignumFormat::Signum3(f) => f.extension(),
        }
    }

    fn file_format_name(&self) -> &'static str {
        match self {
            SignumFormat::Signum1(f) => f.file_format_name(),
            SignumFormat::Signum3(f) => f.file_format_name(),
        }
    }
}
