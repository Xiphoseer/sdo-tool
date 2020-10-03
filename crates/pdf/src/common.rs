use std::{collections::BTreeMap, io, ops::Add, ops::Deref, ops::DerefMut, ops::Mul};

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

impl Rectangle<i32> {
    /// The media box for A4 Paper (Portrait)
    pub fn a4_media_box() -> Self {
        Rectangle {
            ll: Point { x: 0, y: 0 },
            ur: Point { x: 592, y: 842 },
        }
    }
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

/// A font matrix
///
/// <pre style="line-height: 120%;">
/// ⎛ a b 0 ⎞
/// ⎜ c d 0 ⎟
/// ⎝ e f 1 ⎠
/// </pre>
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Matrix<P> {
    pub a: P,
    pub b: P,
    pub c: P,
    pub d: P,
    pub e: P,
    pub f: P,
}

impl<P> Mul for Matrix<P>
where
    P: Copy + Mul<Output = P> + Add<Output = P>,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            a: self.a * rhs.a + self.b * rhs.c,
            b: self.a * rhs.b + self.b * rhs.d,
            c: self.c * rhs.a + self.d * rhs.c,
            d: self.c * rhs.b + self.d * rhs.d,
            e: self.e * rhs.a + self.f * rhs.c + rhs.e,
            f: self.e * rhs.b + self.f * rhs.d + rhs.f,
        }
    }
}

/// A matrix
impl Matrix<f32> {
    /// ```
    /// use pdf_create::common::Matrix;
    /// let id = Matrix::<f32>::identity();
    /// assert_eq!(id, id * id);
    /// ```
    pub fn identity() -> Self {
        Self::scale(1.0, 1.0)
    }

    /// ```
    /// use pdf_create::common::Matrix;
    /// let id = Matrix::<f32>::identity();
    /// let ivy = Matrix::<f32>::inverse_y();
    /// assert_eq!(ivy, ivy * id);
    /// assert_eq!(id, ivy * ivy);
    /// ```
    pub fn inverse_y() -> Self {
        Self::scale(1.0, -1.0)
    }

    /// ```
    /// use pdf_create::common::Matrix;
    /// let id = Matrix::<f32>::identity();
    /// let ivx = Matrix::<f32>::inverse_x();
    /// assert_eq!(ivx, ivx * id);
    /// assert_eq!(id, ivx * ivx);
    /// ```
    pub fn inverse_x() -> Self {
        Self::scale(-1.0, 1.0)
    }

    pub fn inverse_xy() -> Self {
        Self::scale(-1.0, -1.0)
    }

    /// ```
    /// use pdf_create::common::Matrix;
    /// let ty1 = Matrix::<f32>::translate(0.0, 3.0);
    /// let ty2 = Matrix::<f32>::translate(0.0, 5.0);
    /// let tx1 = Matrix::<f32>::translate(2.0, 0.0);
    /// let tx2 = Matrix::<f32>::translate(7.0, 0.0);
    /// let res = Matrix::<f32>::translate(9.0, 8.0);
    /// assert_eq!(res, ty1 * ty2 * tx1 * tx2);
    /// assert_eq!(res, ty1 * tx2 * ty2 * tx1);
    /// ```
    pub fn translate(x: f32, y: f32) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: x,
            f: y,
        }
    }

    pub fn scale(x: f32, y: f32) -> Self {
        Self {
            a: x,
            b: 0.0,
            c: 0.0,
            d: y,
            e: 0.0,
            f: 0.0,
        }
    }

    pub fn default_glyph() -> Self {
        Self::scale(0.0001, 0.0001)
    }
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