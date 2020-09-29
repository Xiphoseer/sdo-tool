use std::{collections::BTreeMap, io, ops::Deref, ops::DerefMut};

use crate::write::{Formatter, PdfName, Serialize};

#[derive(Debug, Copy, Clone)]
pub enum BaseEncoding {
    MacRomanEncoding,
    WinAnsiEncoding,
    MacExpertEncoding,
}

impl Serialize for BaseEncoding {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        match self {
            Self::MacRomanEncoding => PdfName("MacRomanEncoding").write(f),
            Self::WinAnsiEncoding => PdfName("WinAnsiEncoding").write(f),
            Self::MacExpertEncoding => PdfName("MacExpertEncoding").write(f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SparseSet<T> {
    inner: Vec<Option<T>>,
}

impl<T> Default for SparseSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for SparseSet<T> {
    type Target = Vec<Option<T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for SparseSet<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> SparseSet<T> {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }
}

impl<T: Clone> SparseSet<T> {
    pub fn with_size(size: usize) -> Self {
        Self {
            inner: vec![None; size],
        }
    }
}

impl<T: Serialize> Serialize for SparseSet<T> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        let mut arr = f.pdf_arr();
        let mut needs_number = true;
        for (index, entry) in self.inner.iter().enumerate() {
            if let Some(value) = entry {
                if needs_number {
                    arr.entry(&index)?;
                    needs_number = false;
                }
                arr.entry(value)?;
            } else {
                needs_number = true;
            }
        }
        arr.finish()
    }
}

#[derive(Debug, Clone)]
pub struct Encoding<'a> {
    pub base_encoding: Option<BaseEncoding>,
    pub differences: Option<SparseSet<PdfName<'a>>>,
}

impl Serialize for Encoding<'_> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_dict()
            .field("Type", &PdfName("Encoding"))?
            .opt_field("BaseEncoding", &self.base_encoding)?
            .opt_field("Differences", &self.differences)?
            .finish()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Point<P> {
    pub x: P,
    pub y: P,
}

impl<P: Default> Default for Point<P> {
    fn default() -> Self {
        Self {
            x: P::default(),
            y: P::default(),
        }
    }
}

/// A primitive rectangle
#[derive(Debug, Copy, Clone)]
pub struct Rectangle<P> {
    /// lower left
    pub ll: Point<P>,
    /// upper right
    pub ur: Point<P>,
}

impl<P: Serialize> Serialize for Rectangle<P> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_arr()
            .entry(&self.ll.x)?
            .entry(&self.ll.y)?
            .entry(&self.ur.x)?
            .entry(&self.ur.y)?
            .finish()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Matrix<P> {
    pub a: P,
    pub b: P,
    pub c: P,
    pub d: P,
    pub e: P,
    pub f: P,
}

impl<P: Serialize> Serialize for Matrix<P> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_arr()
            .entry(&self.a)?
            .entry(&self.b)?
            .entry(&self.c)?
            .entry(&self.d)?
            .entry(&self.e)?
            .entry(&self.f)?
            .finish()
    }
}

impl Matrix<f32> {
    pub fn default_glyph() -> Self {
        Self {
            a: 0.0001,
            b: 0.0,
            c: 0.0,
            d: 0.0001,
            e: 0.0,
            f: 0.0,
        }
    }
}

pub type Dict<P> = BTreeMap<String, P>;

impl<P: Serialize> Serialize for Dict<P> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        let mut dict = f.pdf_dict();
        for (key, value) in self {
            dict.field(key, value)?;
        }
        dict.finish()
    }
}

pub enum ProcSet {
    PDF,
    Text,
    ImageB,
    ImageC,
    ImageI,
}

impl Serialize for ProcSet {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        match self {
            Self::PDF => PdfName("PDF").write(f),
            Self::Text => PdfName("Text").write(f),
            Self::ImageB => PdfName("ImageB").write(f),
            Self::ImageC => PdfName("ImageC").write(f),
            Self::ImageI => PdfName("ImageI").write(f),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Trapped {
    True,
    False,
    Unknown,
}

impl Default for Trapped {
    fn default() -> Self {
        Trapped::Unknown
    }
}

impl Serialize for Trapped {
    fn write(&self, _f: &mut Formatter) -> io::Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Date {}

impl Serialize for Date {
    fn write(&self, _f: &mut Formatter) -> io::Result<()> {
        todo!()
    }
}
