use std::{fmt, ops::Deref};

use bstr::{BStr, ByteSlice};

use crate::chsets::{printer::PrinterKind, FontKind};

use super::Signum1Format;

/// A four character code
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FourCC(pub(crate) [u8; 4]);

impl FourCC {
    /// `sdoc` - Signum Document
    pub const SDOC: FourCC = FourCC(*b"sdoc");
    /// `bimc` - Hardcopy Image File (standalone)
    pub const BIMC: FourCC = FourCC(*b"bimc");
    /// `eset` - Editor Font
    pub const ESET: FourCC = FourCC(*b"eset");
    /// `ps24` - 24-Needle Printer Font
    pub const PS24: FourCC = FourCC(*b"ps24");
    /// `ps09` - 24-Needle Printer Font
    pub const PS09: FourCC = FourCC(*b"ps09");
    /// `ls30` - 24-Needle Printer Font
    pub const LS30: FourCC = FourCC(*b"ls30");
    /// `sclb` - Signum Clipboard
    pub const SCLB: FourCC = FourCC(*b"sclb");

    /// `0001`
    pub const _0001: FourCC = FourCC(*b"0001");
    /// `cset`
    pub const _CSET: FourCC = FourCC(*b"cset");
    /// `sysp`
    pub const _SYSP: FourCC = FourCC(*b"sysp");
    /// `pbuf`
    pub const _PBUF: FourCC = FourCC(*b"pbuf");
    /// `tebu`
    pub const _TEBU: FourCC = FourCC(*b"tebu");
    /// `hcim`
    pub const _HCIM: FourCC = FourCC(*b"hcim");
    /// `pl01`
    pub const _PL01: FourCC = FourCC(*b"pl01");
    /// `syp2`
    pub const _SYP2: FourCC = FourCC(*b"syp2");

    /// Return this FourCC as a slice of bytes
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Return this FourCC as a [`bstr::BStr`]
    pub fn as_bstr(&self) -> &BStr {
        self.0.as_bstr()
    }

    /// Create a new FourCC
    pub const fn new(buf: [u8; 4]) -> Self {
        Self(buf)
    }

    /// Return the Signum 1 file format identified by this four-character-code, if any
    ///
    /// ```
    /// # use signum::util::FourCC;
    /// # use signum::util::Signum1Format;
    /// assert_eq!(FourCC::SDOC.file_format(), Some(Signum1Format::Document));
    /// ```
    ///
    /// Returns `None` if the format is unknown
    pub const fn file_format(self) -> Option<Signum1Format> {
        use {FontKind::Printer, Signum1Format::Font};
        match self {
            FourCC::SDOC => Some(Signum1Format::Document),
            FourCC::LS30 => Some(Font(Printer(PrinterKind::Laser30))),
            FourCC::PS24 => Some(Font(Printer(PrinterKind::Needle24))),
            FourCC::PS09 => Some(Font(Printer(PrinterKind::Needle9))),
            FourCC::ESET => Some(Font(FontKind::Editor)),
            FourCC::BIMC => Some(Signum1Format::HardcopyImage),
            FourCC::SCLB => Some(Signum1Format::Clipboard),
            _ => None,
        }
    }
}

impl fmt::Debug for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.as_bstr(), f)
    }
}

impl fmt::Display for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.as_bstr(), f)
    }
}

impl Deref for FourCC {
    type Target = [u8; 4];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<FourCC> for Option<Signum1Format> {
    fn from(value: FourCC) -> Self {
        value.file_format()
    }
}

impl From<FourCC> for Option<FontKind> {
    fn from(value: FourCC) -> Self {
        match value {
            FourCC::LS30 => Some(FontKind::Printer(PrinterKind::Laser30)),
            FourCC::PS24 => Some(FontKind::Printer(PrinterKind::Needle24)),
            FourCC::PS09 => Some(FontKind::Printer(PrinterKind::Needle9)),
            FourCC::ESET => Some(FontKind::Editor),
            _ => None,
        }
    }
}

impl From<FourCC> for Option<PrinterKind> {
    fn from(value: FourCC) -> Self {
        match value {
            FourCC::LS30 => Some(PrinterKind::Laser30),
            FourCC::PS24 => Some(PrinterKind::Needle24),
            FourCC::PS09 => Some(PrinterKind::Needle9),
            _ => None,
        }
    }
}
