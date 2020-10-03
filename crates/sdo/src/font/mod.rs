#![allow(dead_code)]
use std::{io, str::FromStr};

use printer::PrinterKind;
use thiserror::*;

pub mod dvips;
pub mod editor;
pub mod encoding;
pub mod printer;

#[derive(Debug, Copy, Clone)]
pub enum FontKind {
    Editor,
    Printer(PrinterKind),
}

impl FontKind {
    pub fn scale_y(&self, units: u16) -> u32 {
        match self {
            Self::Editor => u32::from(units) * 2,
            Self::Printer(PrinterKind::Needle9) => u32::from(units) * 4,
            Self::Printer(PrinterKind::Needle24) => u32::from(units) * 20 / 3,
            Self::Printer(PrinterKind::Laser30) => u32::from(units) * 50 / 9,
        }
    }

    pub fn scale_x(&self, units: u16) -> u32 {
        match self {
            Self::Editor => u32::from(units),
            Self::Printer(pk) => pk.scale_x(units),
        }
    }

    /// Returns the amount of pixels from the top of the box to
    /// the baseline of the font.
    pub fn baseline(&self) -> i16 {
        match self {
            Self::Editor => 18,
            Self::Printer(pk) => pk.baseline(),
        }
    }

    pub fn resolution(&self) -> (isize, isize) {
        match self {
            Self::Editor => (104, 90),
            Self::Printer(PrinterKind::Needle9) => (216, 216),
            Self::Printer(PrinterKind::Needle24) => (360, 360),
            Self::Printer(PrinterKind::Laser30) => (300, 300),
        }
    }

    /// Get the scale that needs to be applied to the font to
    /// get the correct resoltion.
    ///
    /// FIXME: Make this part of the font matrix?
    pub fn scale(&self) -> f32 {
        match self {
            Self::Printer(pk) => pk.scale(),
            Self::Editor => todo!(),
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Editor => "E24",
            Self::Printer(p) => p.extension(),
        }
    }
}

#[derive(Debug, Error)]
#[error("Unknown print driver!")]
pub struct UnknownFontKind {}

impl FromStr for FontKind {
    type Err = UnknownFontKind;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "P09" => Ok(Self::Printer(PrinterKind::Needle9)),
            "E24" => Ok(Self::Editor),
            "P24" => Ok(Self::Printer(PrinterKind::Needle24)),
            "L30" => Ok(Self::Printer(PrinterKind::Laser30)),
            _ => Err(UnknownFontKind {}),
        }
    }
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Failed IO")]
    Io(#[from] io::Error),
    #[error("Unimplemented")]
    Unimplemented,
}
