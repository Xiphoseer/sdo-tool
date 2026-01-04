//! Common structs and enums

use std::{
    collections::BTreeMap,
    fmt, io,
    num::{NonZeroI32, NonZeroU32},
    ops::{Add, Deref, DerefMut, Mul},
    str::FromStr,
};

//use pdf::primitive::PdfString;

use crate::{
    encoding::{pdf_doc_encode, PDFDocEncodingError},
    write::{Formatter, PdfName, PdfNameBuf, Serialize, ToDict},
};

/// A PDF Byte string
#[derive(Clone, Eq, PartialEq)]
pub struct PdfString(Vec<u8>);

impl PdfString {
    /// Create a new string
    pub fn new(string: &[u8]) -> Self {
        Self(string.to_vec())
    }
}

impl FromStr for PdfString {
    type Err = PDFDocEncodingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        pdf_doc_encode(s).map(Self)
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

/// The font stretch value.
///
/// The specific interpretation of these values varies from font to font.
///
/// Example: [`FontStretch::Condensed`] in one font may appear most similar to [`FontStretch::Normal`] in another.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FontStretch {
    /// Ultra Condensed
    UltraCondensed,
    /// Extra Condensed
    ExtraCondensed,
    /// Condensed
    Condensed,
    /// Semi Condensed
    SemiCondensed,
    /// Normal
    Normal,
    /// Semi Expanded
    SemiExpanded,
    /// Expanded
    Expanded,
    /// Extra Expanded
    ExtraExpanded,
    /// Ultra Expanded
    UltraExpanded,
}

impl Serialize for FontStretch {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        match self {
            Self::UltraCondensed => PdfName("UltraCondensed").write(f),
            Self::ExtraCondensed => PdfName("ExtraCondensed").write(f),
            Self::Condensed => PdfName("Condensed").write(f),
            Self::SemiCondensed => PdfName("SemiCondensed").write(f),
            Self::Normal => PdfName("Normal").write(f),
            Self::SemiExpanded => PdfName("SemiExpanded").write(f),
            Self::Expanded => PdfName("Expanded").write(f),
            Self::ExtraExpanded => PdfName("ExtraExpanded").write(f),
            Self::UltraExpanded => PdfName("UltraExpanded").write(f),
        }
    }
}

bitflags::bitflags! {
    /// Font flags specifying various characteristics of the font.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct FontFlags: u32 {
        /// All glyphs have the same width (as opposed to proportional or
        /// variable-pitch fonts, which have different widths)
        const FIXED_PITCH = 1 << 0;
        /// Glyphs have serifs, which are short strokes drawn at an angle on the
        /// top and bottom of glyph stems. (Sans serif fonts do not have serifs.)
        const SERIF = 1 << 1;
        /// Font contains glyphs outside the Adobe standard Latin character set.
        /// This flag and the Nonsymbolic flag shall not both be set or both be
        /// clear
        const SYMBOLIC = 1 << 2;
        /// Glyphs resemble cursive handwriting.
        const SCRIPT = 1 << 3;
        /// Font uses the Adobe standard Latin character set or a subset of it.
        const NONSYMBOLIC = 1 << 5;
        /// Glyphs have dominant vertical strokes that are slanted.
        const ITALIC = 1 << 6;
        /// Font contains no lowercase letters; typically used for display purposes,
        /// such as for titles or headlines.
        const ALL_CAPS = 1 << 16;
        /// Font contains both uppercase and lowercase letters. The uppercase
        /// letters are similar to those in the regular version of the same typeface
        /// family. The glyphs for the lowercase letters have the same shapes as
        /// the corresponding uppercase letters, but they are sized and their
        /// proportions adjusted so that they have the same size and stroke
        /// weight as lowercase glyphs in the same typeface family.
        const SMALL_CAP = 1 << 17;
        /// The ForceBold flag (bit 19) shall determine whether bold glyphs shall be painted with extra pixels even at very
        /// small text sizes by a conforming reader. If the ForceBold flag is set, features of bold glyphs may be thickened at
        /// small text sizes.
        const FORCE_BOLD = 1 << 18;
    }
}

impl Default for FontFlags {
    fn default() -> Self {
        Self::SYMBOLIC
    }
}

impl Serialize for FontFlags {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        self.bits().write(f)
    }
}

/// A font descriptor
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontDescriptor {
    /// **FontName**
    pub font_name: PdfNameBuf,
    /// **FontFamily**
    pub font_family: PdfString,
    /// **FontStretch**
    pub font_stretch: Option<FontStretch>,

    /// **FontWeight**
    ///
    /// The weight (thickness) component of the fully-qualified
    /// font name or font specifier. The possible values shall be 100, 200, 300,
    /// 400, 500, 600, 700, 800, or 900, where each number indicates a
    /// weight that is at least as dark as its predecessor. A value of 400 shall
    /// indicate a normal weight; 700 shall indicate bold.
    /// The specific interpretation of these values varies from font to font.
    ///
    /// (PDF 1.5; should be used for Type 3 fonts in Tagged PDF documents)
    pub font_weight: Option<u16>,

    /// **Flags**: A collection of flags defining various characteristics of the font.
    pub flags: FontFlags,

    /// A rectangle, expressed in the glyph coordinate system, that shall specify the font bounding box.
    ///
    /// This should be the smallest rectangle enclosing the shape that would result if all
    /// of the glyphs of the font were placed with their origins coincident and then filled.
    ///
    /// (Required, except for Type 3 fonts)
    pub font_bbox: Option<Rectangle<i32>>,

    /// **ItalicAngle**: The angle, expressed in degrees counterclockwise from
    /// the vertical, of the dominant vertical strokes of the font.
    ///
    /// The value shall be negative for fonts that slope to the right, as almost all italic fonts do.
    pub italic_angle: i32,

    /// **Ascent**: The maximum height above the
    /// baseline reached by glyphs in this font. The height of glyphs for
    /// accented characters shall be excluded.
    ///
    /// (Required, except for Type 3 fonts)
    pub ascent: Option<i32>,

    /// **Descent**: The maximum depth below the
    /// baseline reached by glyphs in this font. The value shall be a negative
    /// number.
    ///
    /// (Required, except for Type 3 fonts)
    pub descent: Option<i32>,

    /// **Leading**: The spacing between baselines of consecutive lines of text.
    ///
    /// Default value: 0.
    pub leading: Option<NonZeroI32>,

    /// **CapHeight**: The vertical coordinate of the top of flat capital letters, measured from the baseline.
    ///
    /// (Required for fonts that have Latin characters, except for Type 3 fonts)
    pub cap_height: Option<NonZeroU32>,

    /// **XHeight**: The font’s x height: the vertical coordinate of the top of flat
    /// nonascending lowercase letters (like the letter x), measured from the
    /// baseline, in fonts that have Latin characters.
    ///
    /// Default value: 0.
    pub x_height: Option<NonZeroU32>,

    /// **StemV**  The thickness, measured horizontally, of the dominant vertical stems of glyphs in the font.
    ///
    /// (Required, except for Type 3 fonts)
    pub stem_v: Option<NonZeroU32>,

    /// **StemH** The thickness, measured vertically, of the dominant horizontal stems of glyphs
    /// in the font.
    ///
    /// Default value: 0.
    pub stem_h: Option<NonZeroU32>,
}

impl Serialize for FontDescriptor {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        let mut dict = f.pdf_dict();
        dict.field("Type", &PdfName("FontDescriptor"))?
            .field("FontName", &self.font_name.as_ref())?
            .field("FontFamily", &self.font_family)?
            .opt_field("FontStretch", &self.font_stretch)?
            .opt_field("FontWeight", &self.font_weight)?
            .field("Flags", &self.flags)?
            .opt_field("FontBBox", &self.font_bbox)?
            .field("ItalicAngle", &self.italic_angle)?
            .opt_field("Ascent", &self.ascent)?
            .opt_field("Descent", &self.descent)?
            .opt_field("Leading", &self.leading)?
            .opt_field("CapHeight", &self.cap_height)?
            .opt_field("XHeight", &self.x_height)?
            .opt_field("StemV", &self.stem_v)?
            .opt_field("StemH", &self.stem_h)?
            .finish()?;
        Ok(())
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

impl<A, B> Serialize for BTreeSer<'_, A, B>
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

/// A media box definition
#[derive(Debug, Copy, Clone)]
pub struct MediaBox {
    /// The width (in millimeters)
    pub width: i32,
    /// The height (in millimeters)
    pub height: i32,
}

impl MediaBox {
    /// An A4 (portrait) Media Box
    pub const A4: Self = Self {
        width: 592,
        height: 842,
    };

    /// An A4 (landscape) Media Box
    pub const A4_LANDSCAPE: Self = Self::A4.rotate_90();

    /// Rotate the media box 90 degrees
    pub const fn rotate_90(self) -> Self {
        Self {
            width: self.height,
            height: self.width,
        }
    }
}

impl From<MediaBox> for Rectangle<i32> {
    fn from(value: MediaBox) -> Self {
        Rectangle::media_box(value.width, value.height)
    }
}

/// A primitive rectangle
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Rectangle<P> {
    /// lower left
    pub ll: Point<P>,
    /// upper right
    pub ur: Point<P>,
}

impl Rectangle<i32> {
    /// The media box for A4 Paper (Portrait)
    pub fn a4_media_box() -> Self {
        Self::from(MediaBox::A4)
    }

    /// The media box for A4 Paper (Portrait)
    pub fn media_box(width: i32, height: i32) -> Self {
        Rectangle {
            ll: Point { x: 0, y: 0 },
            ur: Point {
                x: width,
                y: height,
            },
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

    #[allow(clippy::suspicious_operation_groupings)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
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

#[derive(Debug, Copy, Clone, PartialEq)]
/// Parameters for the **CalGray** color space
pub struct CalGrayColorSpaceParams {
    /// The white point of the space [X_W, Y_W, Z_W]
    pub white_point: [f32; 3],
    /// The black point of the space [X_B, Y_B, Z_B]
    pub black_point: Option<[f32; 3]>,
    /// A number G defining the gamma for the gray (A) component.
    /// G shall be positive and is generally greater than or equal to 1.
    ///
    /// Default value: 1.
    pub gamma: Option<f32>,
}

/// The D65 white point
pub const CAL_GRAY_D65: CalGrayColorSpaceParams = CalGrayColorSpaceParams {
    white_point: [0.9505, 1.0000, 1.0890],
    black_point: None,
    gamma: None,
};

impl Serialize for CalGrayColorSpaceParams {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_dict()
            .field("WhitePoint", &self.white_point)?
            .opt_field("BlackPoint", &self.black_point)?
            .opt_field("Gamma", &self.gamma)?
            .finish()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// Parameters for the `Lab` color space
pub struct LabColorSpaceParams {
    /// The white point of the space [X_W, Y_W, Z_W]
    pub white_point: [f32; 3],
    /// The black point of the space [X_B, Y_B, Z_B]
    pub black_point: Option<[f32; 3]>,
    /// The range of the space [a_min, a_max, b_min, b_max]
    pub range: [i32; 4],
}

impl Serialize for LabColorSpaceParams {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.pdf_dict()
            .field("WhitePoint", &self.white_point)?
            .opt_field("BlackPoint", &self.black_point)?
            .field("Range", &self.range)?
            .finish()
    }
}

impl Default for LabColorSpaceParams {
    fn default() -> Self {
        Self {
            white_point: [0.9505, 1.0000, 1.0890],
            black_point: None,
            range: [-128, 127, -128, 127],
        }
    }
}

/// The color space of an image
#[derive(Debug, Copy, Clone, PartialEq)]
#[non_exhaustive]
#[allow(clippy::upper_case_acronyms)]
pub enum ColorSpace {
    /// A 1-component grayscale image
    DeviceGray,
    /// A 3-component RGB image
    DeviceRGB,
    /// A 4-component CMYK image
    DeviceCMYK,
    /// CalGray color space
    CalGray(CalGrayColorSpaceParams),
    /// A L*a*b color space
    Lab(LabColorSpaceParams),
}

impl Serialize for ColorSpace {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        match self {
            Self::DeviceGray => PdfName("DeviceGray").write(f),
            Self::DeviceRGB => PdfName("DeviceRGB").write(f),
            Self::DeviceCMYK => PdfName("DeviceCMYK").write(f),
            Self::CalGray(params) => (PdfName("CalGray"), params).write(f),
            Self::Lab(params) => (PdfName("Lab"), params).write(f),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
/// Specifies which value draws a color for an `ImageMask`
///
/// This serializes to the array parameter for `Decode`
pub enum ColorIs {
    /// 0 means the color is drawn (default)
    #[default]
    Zero,
    /// 1 means the color is drawn
    One,
}

impl Serialize for ColorIs {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        let (color, no) = match self {
            Self::Zero => (0, 1),
            Self::One => (1, 0),
        };
        f.pdf_arr().entry(&color)?.entry(&no)?.finish()
    }
}

/// The metadata for an image XObject
#[derive(Debug, Copy, Clone)]
pub struct ImageMetadata {
    /// The width of the image
    pub width: usize,
    /// The height of the image
    pub height: usize,
    /// The `ColorSpace`
    pub color_space: ColorSpace,
    /// The `BitsPerComponent`
    pub bits_per_component: u8,
    /// The `ImageMask`
    pub image_mask: bool,
    /// The `Decode`
    pub decode: ColorIs,
}

impl ToDict for ImageMetadata {
    fn write(&self, dict: &mut crate::write::PdfDict<'_, '_>) -> io::Result<()> {
        dict.field("Type", &PdfName("XObject"))?;
        dict.field("Subtype", &PdfName("Image"))?;
        dict.field("Width", &self.width)?;
        dict.field("Height", &self.height)?;
        dict.field("ColorSpace", &self.color_space)?;
        dict.field("BitsPerComponent", &self.bits_per_component)?;
        dict.default_field("Decode", &self.decode)?;
        if self.image_mask {
            dict.field("ImageMask", &true)?;
        }
        Ok(())
    }
}

/// The metadata for a stream
#[derive(Debug, Copy, Clone)]
pub enum StreamMetadata {
    /// No specific metadata (e.g. `CharProc`)
    None,
    /// Metadata for an Image
    Image(ImageMetadata),
    /// Metadata for a color Profile
    ColorProfile(ICCColorProfileMetadata),
    /// Metadata
    MetadataXML,
}

impl ToDict for StreamMetadata {
    fn write(&self, dict: &mut crate::write::PdfDict<'_, '_>) -> io::Result<()> {
        match self {
            Self::None => Ok(()),
            Self::Image(i) => i.write(dict),
            Self::ColorProfile(m) => m.write(dict),
            Self::MetadataXML => {
                dict.field("Type", &PdfName("Metadata"))?;
                dict.field("Subtype", &PdfName("XML"))?;
                Ok(())
            }
        }
    }
}

/// Color Profile metadata
#[derive(Debug, Copy, Clone)]
pub struct ICCColorProfileMetadata {
    /// An alternate color space
    pub alternate: Option<ColorSpace>,
    /// Number of color components: 1, 3, or 4
    pub num_components: u8,
}

impl ToDict for ICCColorProfileMetadata {
    fn write(&self, dict: &mut crate::write::PdfDict<'_, '_>) -> io::Result<()> {
        dict.field("N", &self.num_components)?;
        dict.opt_field("Alternate", &self.alternate)?;
        Ok(())
    }
}

/// Indicates whether the PDF is trapped
#[derive(Debug, PartialEq, Eq, Default)]
#[allow(missing_docs)]
pub enum Trapped {
    True,
    False,
    #[default]
    Unknown,
}

impl Serialize for Trapped {
    fn write(&self, _f: &mut Formatter) -> io::Result<()> {
        todo!()
    }
}

#[allow(missing_docs, non_camel_case_types, clippy::upper_case_acronyms)]
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
pub struct OutputIntent<Profile> {
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
    /// Output profile stream
    pub dest_output_profile: Option<Profile>,
}

impl Serialize for OutputIntent<ObjRef> {
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
            .opt_field("DestOutputProfile", &self.dest_output_profile)?
            .finish()
    }
}
