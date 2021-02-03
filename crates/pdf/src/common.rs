//! Common structs and enums

use std::{
    collections::BTreeMap,
    fmt, io,
    ops::{Add, Deref, DerefMut, Mul},
};

//use pdf::primitive::PdfString;

use crate::write::{Formatter, PdfName, Serialize};

/// A PDF Byte string
#[derive(Clone, Eq, PartialEq)]
pub struct PdfString(Vec<u8>);

impl PdfString {
    /// Create a new string
    pub fn new<S: AsRef<[u8]>>(string: S) -> Self {
        Self(string.as_ref().to_vec())
    }
}

impl PdfString {
    /// Get a slice to the contained bytes
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }

    /// Get the contained byte buffer
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl fmt::Debug for PdfString {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

/// A reference to an object
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ObjRef {
    /// The index within the file
    pub id: u64,
    /// The generation number
    pub gen: u16,
}

/// The base encoding for a font
#[derive(Debug, Copy, Clone)]
pub enum BaseEncoding {
    /// `MacRomanEncoding`
    MacRomanEncoding,
    /// `WinAnsiEncoding`
    WinAnsiEncoding,
    /// `MacExpertEncoding`
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

/// The style of a page label number
#[derive(Debug, Clone)]
pub enum PageLabelKind {
    /// Arabic decimal numerals (1, 2, 3, 4, …)
    Decimal,
    /// Lowercase roman numerals (i, ii, iii, iv, …)
    RomanLower,
    /// Uppercase roman numerals (I, II, III, IV, …)
    RomanUpper,
    /// Lowercase letters (a-z, aa-zz, …)
    AlphaLower,
    /// Lowercase letters (a-z, aa-zz, …)
    AlphaUpper,
}

impl Serialize for PageLabelKind {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        match self {
            Self::Decimal => PdfName("D").write(f),
            Self::RomanLower => PdfName("r").write(f),
            Self::RomanUpper => PdfName("R").write(f),
            Self::AlphaLower => PdfName("a").write(f),
            Self::AlphaUpper => PdfName("A").write(f),
        }
    }
}

/// Specification for the labels of a sequence of pages
#[derive(Debug, Clone)]
pub struct PageLabel {
    /// Fixed string prepended to every number
    pub prefix: PdfString,
    /// The style of the number
    pub kind: Option<PageLabelKind>,
    /// The value for the number on the first page of the group
    pub start: u32,
}

impl Serialize for PageLabel {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        let mut dict = f.pdf_dict();
        dict.field("Type", &PdfName("PageLabel"))?;
        dict.opt_field("S", &self.kind)?;
        dict.field("St", &self.start)?;
        if !self.prefix.as_bytes().is_empty() {
            dict.field("P", &self.prefix)?;
        }
        dict.finish()
    }
}

struct BTreeSer<'a, A, B>(&'a BTreeMap<A, B>);

impl<'a, A, B> Serialize for BTreeSer<'a, A, B>
where
    A: Serialize,
    B: Serialize,
{
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        let mut arr = f.pdf_arr();
        for (key, value) in self.0 {
            arr.entry(key)?;
            arr.entry(value)?;
        }
        arr.finish()
    }
}

/// A tree of numbers
pub struct NumberTree<T> {
    inner: BTreeMap<usize, T>,
}

impl<T> Default for NumberTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<BTreeMap<usize, T>> for NumberTree<T> {
    fn from(tree: BTreeMap<usize, T>) -> Self {
        Self { inner: tree }
    }
}

impl<T> NumberTree<T> {
    /// Creates a new tree
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    /// Checks whether the tree is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Inserts a node into the tree
    pub fn insert(&mut self, key: usize, value: T) -> Option<T> {
        self.inner.insert(key, value)
    }
}

impl<T: Serialize> Serialize for NumberTree<T> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_dict().field("Nums", &BTreeSer(&self.inner))?.finish()
    }
}

#[derive(Debug, Clone)]
/// A vector of options
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
    /// Creates a new sparse set
    pub fn new() -> Self {
        Self { inner: vec![] }
    }
}

impl<T: Clone> SparseSet<T> {
    /// Creates a new set
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

/// A font encoding
#[derive(Debug, Clone)]
pub struct Encoding<'a> {
    /// The base encoding
    pub base_encoding: Option<BaseEncoding>,
    /// The differences from the base encoding
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

/// A simple two-dimensional coordinate
#[derive(Debug, Copy, Clone)]
pub struct Point<P> {
    /// Horizontal offset
    pub x: P,
    /// Vertical offset
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
    /// M<sub>1,1</sub>
    pub a: P,
    /// M<sub>1,2</sub>
    pub b: P,
    /// M<sub>2,1</sub>
    pub c: P,
    /// M<sub>2,2</sub>
    pub d: P,
    /// M<sub>3,1</sub>
    pub e: P,
    /// M<sub>3,2</sub>
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

    /// Inverts both coordinates
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

    /// Create a scaling matrix
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

    /// Create a default 1:1000 glyph matrix
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

/// A dict is a map from strings to a type P
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

/// Valid `ProcSet`s for PDF files
pub enum ProcSet {
    /// General PDFs procs
    PDF,
    /// Text procs
    Text,
    /// Black/White Images
    ImageB,
    /// Color Images
    ImageC,
    /// TODO: Check Docs
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

/// Indicates whether the PDF is trapped
#[derive(Debug, PartialEq, Eq)]
#[allow(missing_docs)]
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

#[allow(missing_docs, non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum OutputIntentSubtype {
    GTS_PDFX,
    GTS_PDFA1,
    ISO_PDFE1,
}

impl Serialize for OutputIntentSubtype {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        match self {
            Self::GTS_PDFX => PdfName("GTS_PDFX").write(f),
            Self::GTS_PDFA1 => PdfName("GTS_PDFA1").write(f),
            Self::ISO_PDFE1 => PdfName("ISO_PDFE1").write(f),
        }
    }
}

#[derive(Debug, Clone)]
/// An output intent
pub struct OutputIntent {
    /// The subtype / spec
    pub subtype: OutputIntentSubtype,
    /// ???
    pub output_condition: Option<PdfString>,
    /// ???
    pub output_condition_identifier: PdfString,
    /// ???
    pub registry_name: Option<PdfString>,
    /// ???
    pub info: Option<PdfString>,
    // TODO: DestOutputProfile stream
}

impl Serialize for OutputIntent {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_dict()
            .field("Type", &PdfName("OutputIntent"))?
            .field("S", &self.subtype)?
            .opt_field("OutputCondition", &self.output_condition)?
            .field(
                "OutputConditionIdentifier",
                &self.output_condition_identifier,
            )?
            .opt_field("RegistryName", &self.registry_name)?
            .opt_field("Info", &self.info)?
            .finish()
    }
}
