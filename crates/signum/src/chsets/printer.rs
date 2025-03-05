//! # The printer charsets

use super::{metrics::BBox, Device, FontResolution, LoadError};
use crate::{
    docs::four_cc,
    raster::Page,
    util::{Buf, FileFormatKind, FourCC},
};
use nom::{
    bytes::complete::{tag, take},
    combinator::{cond, verify},
    error::{ErrorKind, ParseError},
    multi::count,
    number::complete::{be_u32, u8},
    Finish, IResult,
};
use std::{
    borrow::Cow,
    convert::TryInto,
    fmt,
    io::{self, Write},
    num::{NonZero, NonZeroU8, TryFromIntError},
    ops::Range,
    path::Path,
};

#[derive(Debug, Copy, Clone)]
/// The supported kinds of printers
pub enum PrinterKind {
    /// A 24-needle printer
    Needle24,
    /// A 9-needle printer
    Needle9,
    /// A laser printer
    Laser30,
}

impl Device for PrinterKind {
    fn resolution(&self) -> &'static FontResolution {
        match self {
            Self::Needle9 => &FontResolution { x: 240, y: 216 },
            Self::Needle24 => &FontResolution { x: 360, y: 360 },
            Self::Laser30 => &FontResolution { x: 300, y: 300 },
        }
    }
}

impl FileFormatKind for PrinterKind {
    /// Get the extension used for charset files for this printer kind
    fn extension(&self) -> &'static str {
        match self {
            Self::Needle24 => "P24",
            Self::Needle9 => "P9",
            Self::Laser30 => "L30",
        }
    }

    /// Get the extension used for charset files for this printer kind
    fn magic(&self) -> FourCC {
        match self {
            Self::Needle24 => FourCC::PS24,
            Self::Needle9 => FourCC::PS09,
            Self::Laser30 => FourCC::LS30,
        }
    }

    /// Get the file format name for this printer kind
    fn file_format_name(&self) -> &'static str {
        match self {
            Self::Needle24 => "Signum! 24-Needle Printer Bitmap Font",
            Self::Needle9 => "Signum! 9-Needle Printer Bitmap Font",
            Self::Laser30 => "Signum! Laser Printer Bitmap Font",
        }
    }
}

impl PrinterKind {
    /// Get the number of dots for the given amount of horizontal units
    ///
    /// FIXME: this introduces rounding errors
    pub fn scale_x(self, units: u16) -> u32 {
        match self {
            Self::Needle9 => u32::from(units) * 12 / 5,
            Self::Needle24 => u32::from(units) * 4,
            Self::Laser30 => u32::from(units) * 10 / 3,
        }
    }

    /// Get the number of dots for the given amount of vertical units
    ///
    /// FIXME: this introduces rounding errors
    pub fn scale_y(&self, units: u16) -> u32 {
        match self {
            PrinterKind::Needle9 => u32::from(units) * 4,
            PrinterKind::Needle24 => u32::from(units) * 20 / 3,
            PrinterKind::Laser30 => u32::from(units) * 50 / 9,
        }
    }

    /// Get the position of the character baseline from the top of the glyph bounding box
    pub fn baseline(self) -> u32 {
        match self {
            Self::Needle9 => 35,
            Self::Needle24 => 58,
            Self::Laser30 => 48,
        }
    }

    /// Get maximum height of a character, as indicated by the font editors
    pub fn max_height(self) -> u32 {
        match self {
            Self::Needle9 => 48,
            Self::Needle24 => 80,
            Self::Laser30 => 68,
        }
    }

    /// The space between the baseline and the bottom of the bounding box (in pixels)
    ///
    /// ```
    /// # use signum::chsets::printer::PrinterKind;
    /// let (p9, p24, l30) = (PrinterKind::Needle9, PrinterKind::Needle24, PrinterKind::Laser30);
    /// assert_eq!(p9.max_descent(), p9.max_height() - p9.baseline());
    /// assert_eq!(p24.max_descent(), p24.max_height() - p24.baseline());
    /// assert_eq!(l30.max_descent(), l30.max_height() - l30.baseline());
    /// ```
    pub fn max_descent(self) -> u32 {
        match self {
            Self::Needle9 => 13,
            Self::Needle24 => 22,
            Self::Laser30 => 20,
        }
    }

    /// Distance between the baseline and the top of the bounding box (in pixels)
    ///
    /// Alias for [`baseline`][Self::baseline]
    ///
    /// ```
    /// # use signum::chsets::printer::PrinterKind;
    /// let (p9, p24, l30) = (PrinterKind::Needle9, PrinterKind::Needle24, PrinterKind::Laser30);
    /// assert_eq!(p9.max_height(), p9.max_ascent() + p9.max_descent());
    /// assert_eq!(p24.max_height(), p24.max_ascent() + p24.max_descent());
    /// assert_eq!(l30.max_height(), l30.max_ascent() + l30.max_descent());
    /// ```
    pub fn max_ascent(self) -> u32 {
        self.baseline()
    }

    /// The default height of the character box (in pixels)
    ///
    /// This is the distance from the baseline to the dashed-line indicating
    /// the "default" cap height in the signum font editors.
    pub fn reference_cap_height(self) -> u32 {
        match self {
            Self::Needle9 => 22,
            Self::Needle24 => 36,
            Self::Laser30 => 30,
        }
    }
}

impl fmt::Display for PrinterKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.file_format_name().fmt(f)
    }
}

#[derive(Debug)]
/// A complete printer charset
pub struct PSet<'a> {
    /// The kind
    pub pk: PrinterKind,
    /// The header
    pub header: Buf<'a>,
    /// The list of characters
    pub chars: Vec<PSetChar<'a>>,
}

impl PSet<'_> {
    /// Get the bounding box of pixels in this font
    pub fn font_bbox(&self) -> BBox {
        let mut bbox = BBox {
            ll_x: usize::MAX,
            ll_y: i32::MAX,
            ur_x: usize::MIN,
            ur_y: i32::MIN,
        };
        for chr in &self.chars {
            if let Some(hb) = chr.hbounds() {
                let (lx, rx) = hb.left_right_x();
                bbox.ll_x = bbox.ll_x.min(lx);
                bbox.ur_x = bbox.ur_x.max(rx);
                let vb = chr.vbounds(self.pk.baseline());
                bbox.ll_y = bbox.ll_y.min(vb.start);
                bbox.ur_y = bbox.ur_y.max(vb.end);
            }
        }
        bbox
    }

    /// Write printer font to a file
    pub fn write_to<W: Write>(&self, buf: &mut W) -> io::Result<()> {
        buf.write_all(self.pk.magic().as_slice())?;
        buf.write_all(b"0001")?;
        buf.write_all(&128u32.to_be_bytes())?;
        for _ in 0..32 {
            buf.write_all(&[0, 0, 0, 0])?;
        }
        let mut off: u32 = 4;
        let mut offsets = Vec::with_capacity(128);
        for i in 1..128 {
            offsets.push(off);
            let c = &self.chars[i];
            off += (c.width as u32 * c.height as u32) + 4;
            if off % 2 == 1 {
                off += 1;
            }
        }
        let max: u32 = off;
        buf.write_all(&max.to_be_bytes())?;
        for off in offsets {
            buf.write_all(&off.to_be_bytes())?;
        }
        for i in 0..128 {
            let c = &self.chars[i];
            let len = c.height as u32 * c.width as u32;
            buf.write_all(&[c.top, c.height, c.width, c._d])?;
            assert_eq!(c.bitmap.len() as u32, c.height as u32 * c.width as u32);
            buf.write_all(c.bitmap.as_ref())?;
            if len % 2 == 1 {
                buf.write_all(&[0u8])?;
            }
        }
        Ok(())
    }
}

/// A struct to hold information on computed character dimensions
pub struct HBounds {
    /// The number of bits that are zero in every line from the left
    pub max_lead: usize,
    /// The number of bits that are zero in every line from the right
    pub max_tail: usize,
    /// The number of bytes in the glyph
    pub width: usize,
}

impl HBounds {
    /// Get the right edge position
    pub fn right_x(&self) -> usize {
        self.width * 8 - self.max_tail
    }

    /// Get the left edge position
    pub fn left_x(&self) -> usize {
        self.max_lead
    }

    /// Get the actual width in bits of the glyph
    pub fn bit_width(&self) -> usize {
        self.right_x() - self.left_x()
    }

    /// Get the positions of the minimal left and right edge containing all pixels that are set
    pub fn left_right_x(&self) -> (usize, usize) {
        (self.left_x(), self.right_x())
    }
}

#[derive(Debug, Clone, PartialEq)]
/// A single printer character
pub struct PSetChar<'a> {
    /// The distance to the top of the line box
    pub top: u8,
    /// The height of the character in pixels
    pub height: u8,
    /// The width of the character in bytes
    pub width: u8,
    /// Some unknown property
    _d: u8,
    /// The pixel data
    pub bitmap: Cow<'a, [u8]>,
}

/// An owned version of a PSetChar
pub type OwnedPSetChar = PSetChar<'static>;

impl<'a> PSetChar<'a> {
    /// Create a new instance
    ///
    /// Panics if the width and height don't match the bitmap
    pub fn new(width: u8, height: u8, top: u8, bitmap: &'a [u8]) -> Self {
        assert_eq!(bitmap.len(), width as usize * height as usize);
        Self {
            width,
            height,
            top,
            _d: 0,
            bitmap: Cow::Borrowed(bitmap),
        }
    }

    /// Create a printer char from a [Page]
    pub fn from_page(top: u8, bitmap: Page) -> Result<Self, TryFromIntError> {
        let width = bitmap.bytes_per_line().try_into()?;
        let height = bitmap.bit_height().try_into()?;
        let data = bitmap.into_vec();
        assert_eq!(data.len(), width as usize * height as usize);
        Ok(Self {
            width,
            height,
            top,
            _d: 0,
            bitmap: Cow::Owned(data),
        })
    }
}

impl PSetChar<'static> {
    /// Empty printer char
    pub const EMPTY: Self = Self {
        top: 0,
        height: 0,
        width: 0,
        _d: 0,
        bitmap: Cow::Owned(Vec::new()),
    };
}

impl PSetChar<'_> {
    /// Create an owned version of this character
    pub fn owned(&self) -> OwnedPSetChar {
        PSetChar {
            top: self.top,
            height: self.height,
            width: self.width,
            _d: self._d,
            bitmap: Cow::Owned(self.bitmap.to_vec()),
        }
    }

    /// Return the value of the 'special' 4th byte in the header
    /// of the printer char that is almost always 0
    pub fn special(&self) -> Option<NonZeroU8> {
        NonZero::new(self._d)
    }

    /// Compute the vertical bounds of the char
    ///
    /// This assumes that blank lines are not encode in the bitmap for now
    pub fn vbounds(&self, baseline: u32) -> Range<i32> {
        let sig_origin_y = baseline as i32;
        let sig_upper_y = sig_origin_y - self.top as i32;
        let sig_lower_y = sig_upper_y - self.height as i32;
        sig_lower_y..sig_upper_y
    }

    /// Compute the horizontal bounds of the char
    pub fn hbounds(&self) -> Option<HBounds> {
        if self.width == 0 {
            return None;
        }
        let width = self.width as usize * 8;
        let mut max_lead = width;
        let mut max_tail = width;
        for row in self.bitmap.chunks(self.width as usize) {
            let mut has_bit = false;
            let mut lead: usize = 0;
            let mut tail: usize = 0;
            for byte in row {
                match (has_bit, *byte == 0) {
                    (false, false) => {
                        lead += byte.leading_zeros() as usize;
                        has_bit = true;
                        tail = byte.trailing_zeros() as usize;
                    }
                    (false, true) => {
                        lead += 8;
                    }
                    (true, false) => {
                        tail = byte.trailing_zeros() as usize;
                    }
                    (true, true) => {
                        tail += 8;
                    }
                }
            }
            max_lead = max_lead.min(lead);
            max_tail = max_tail.min(tail);
        }
        Some(HBounds {
            max_lead,
            max_tail,
            width: self.width as usize,
        })
    }

    /// Check whether the given pixel position in the actual bitmap
    /// (which may be smaller than the full bbox) is set to 1 / ink.
    ///
    /// ```
    /// use signum::chsets::printer::PSetChar;
    ///
    /// let p = PSetChar::new(1, 8, 10, &[
    ///     0b10000000,
    ///     0b01000000,
    ///     0b00100000,
    ///     0b00010000,
    ///     0b00001000,
    ///     0b00000100,
    ///     0b00000010,
    ///     0b00000001,
    /// ]);
    /// for i in 0..8 {
    ///     for j in 0..8 {
    ///         assert_eq!(p.get_ink_at(i, j), i == j);
    ///     }
    /// }
    /// ```
    pub fn get_ink_at(&self, x: usize, y: usize) -> bool {
        let offset = x / 8;
        let wu = self.width as usize;
        if offset >= wu {
            return false;
        }
        if y >= self.height as usize {
            return false;
        }
        let byte = self.bitmap[y * wu + offset];
        0x80 & (byte << (x % 8)) > 0
    }

    /// Apply bold mode "light", a 3x1 kernel
    ///
    /// ```
    /// # use signum::chsets::printer::PSetChar;
    /// let ch = PSetChar::new(1, 7, 10, &[
    ///     0b00111000,
    ///     0b00011100,
    ///     0b00001100,
    ///     0b00001100,
    ///     0b00001100,
    ///     0b00011100,
    ///     0b00111000,
    /// ]);
    /// let bch = ch.bold_light();
    /// assert_eq!(bch.bitmap.as_ref(), &[
    ///     0b01111100,
    ///     0b00111110,
    ///     0b00011110,
    ///     0b00011110,
    ///     0b00011110,
    ///     0b00111110,
    ///     0b01111100,
    /// ]);
    /// ```
    pub fn bold_light(&self) -> PSetChar<'static> {
        let mut bitmap = Vec::with_capacity(self.bitmap.len());
        if self.width > 0 {
            for row in self.bitmap.chunks(self.width as usize) {
                let mut acc = 0;
                for slice in row.windows(2) {
                    let a = slice[0];
                    let b = slice[1];
                    bitmap.push(acc | a << 1 | a | a >> 1 | b >> 7);
                    acc = a << 7;
                }
                if let Some(c) = row.last().copied() {
                    bitmap.push(acc | c << 1 | c | c >> 1)
                }
            }
        }
        PSetChar {
            top: self.top,
            height: self.height,
            width: self.width,
            _d: self._d,
            bitmap: Cow::Owned(bitmap),
        }
    }

    /// Apply bold "light" vertically, a 1x3 kernel
    ///
    /// ```
    /// # use signum::chsets::printer::PSetChar;
    /// let ch = PSetChar::new(1, 7, 10, &[
    ///     0b00111000,
    ///     0b00011100,
    ///     0b00001100,
    ///     0b00001100,
    ///     0b00001100,
    ///     0b00011100,
    ///     0b00111000,
    /// ]);
    /// let bch = ch.bold_light_vertical();
    /// assert_eq!(bch.top, 9);
    /// assert_eq!(bch.bitmap.as_ref(), &[
    ///     0b00111000,
    ///     0b00111100,
    ///     0b00111100,
    ///     0b00011100,
    ///     0b00001100,
    ///     0b00011100,
    ///     0b00111100,
    ///     0b00111100,
    ///     0b00111000,
    /// ]);
    /// ```
    pub fn bold_light_vertical(&self) -> PSetChar<'static> {
        let mut bitmap = self.bitmap.to_vec();
        let w = self.width as usize;
        let w2 = w * 2;
        let (dc, dy) = if self.top > 0 { (w2, 2) } else { (w, 1) };
        if w > 0 {
            bitmap.resize(bitmap.len() + dc, 0);
            for (src, dst) in self.bitmap.iter().zip(bitmap[w..].iter_mut()) {
                *dst |= *src
            }
            if self.top > 0 {
                for (src, dst) in self.bitmap.iter().zip(bitmap[w2..].iter_mut()) {
                    *dst |= *src
                }
            }
        }
        PSetChar {
            top: self.top.saturating_sub(1),
            height: self.height + dy,
            width: self.width,
            _d: self._d,
            bitmap: Cow::Owned(bitmap),
        }
    }

    /// Apply the "normal" bold modifier, a 3x3 kernel
    pub fn bold_normal(&self) -> PSetChar<'static> {
        self.bold_light().bold_light_vertical()
    }

    /// Apply the "strong" bold modifier, a 5x3 kernel
    pub fn bold_strong(&self) -> PSetChar<'static> {
        self.bold_normal().bold_light()
    }
}

/// An owned printer character set
pub struct OwnedPSet {
    inner: PSet<'static>,
    #[allow(unused)]
    buffer: Vec<u8>,
}

/*
impl Deref for OwnedPSet {
    type Target = PSet<'static>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
 */

impl<'a> OwnedPSet {
    /// Get a borrowed version of this Owned Set
    pub fn borrowed(&'a self) -> &'a PSet<'a> {
        &self.inner
    }
}

impl OwnedPSet {
    /// Load a character set
    pub fn load(path: &Path, kind: PrinterKind) -> Result<Self, LoadError> {
        let buffer = std::fs::read(path)?;
        Self::load_from_buffer(buffer, kind)
    }

    /// Load a character set from a byte buffer
    pub fn load_from_buffer(buffer: Vec<u8>, kind: PrinterKind) -> Result<Self, LoadError> {
        // SAFETY: this is safe, because `buffer` is plain data and
        // drop order between `inner` and `buffer` really doesn't matter.
        let input: &'static [u8] = unsafe { std::mem::transmute(&buffer[..]) };
        let (_, inner) = match kind {
            PrinterKind::Needle24 => parse_ps24(input),
            PrinterKind::Laser30 => parse_ls30(input),
            PrinterKind::Needle9 => parse_ps09(input),
        }
        .finish()
        .map_err(|e: nom::error::Error<&[u8]>| LoadError::Parse(format!("{:?}", e)))?;
        Ok(Self { inner, buffer })
    }
}

/// Parse a single P24 character
pub fn parse_char<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], PSetChar<'a>, E> {
    let (input, top) = u8(input)?;
    let (input, height) = u8(input)?;
    let (input, width) = u8(input)?;
    // FIXME: Are there any valid files where this is non-zero?
    let (input, _d) = u8(input)?; // verify(u8, |x| *x == 0)(input)?;

    let len = (width as usize) * (height as usize);
    let (input, bitmap) = take(len)(input)?;
    let (input, _a) = cond(len % 2 == 1, u8)(input)?;

    Ok((
        input,
        PSetChar {
            top,
            height,
            width,
            _d,
            bitmap: Cow::Borrowed(bitmap),
        },
    ))
}

/// Parse a a font file
///
/// This method only checks the `0001` part of the magic bytes
pub fn parse_font<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
    pk: PrinterKind,
) -> IResult<&'a [u8], PSet<'a>, E> {
    let (input, _) = tag(b"0001")(input)?;
    let (input, _) = verify(be_u32, |x| *x == 128)(input)?;

    let (input, header) = take(128usize)(input)?;
    let (input, _len) = be_u32(input)?;

    let (input, _offset_buf) = take(127usize * 4)(input)?;
    let (input, chars) = count(parse_char, 128usize)(input)?;

    Ok((
        input,
        PSet {
            pk,
            header: Buf(header),
            chars,
        },
    ))
}

/// Parse a P24 file
pub fn parse_ps24<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], PSet<'a>, E> {
    let (input, _) = tag(b"ps24")(input)?;
    parse_font(input, PrinterKind::Needle24)
}

/// Parse a P09 file
pub fn parse_ps09<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], PSet<'a>, E> {
    let (input, _) = tag(b"ps09")(input)?;
    parse_font(input, PrinterKind::Needle9)
}

/// Parse a L30 file
pub fn parse_ls30<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], PSet<'a>, E> {
    let (input, _) = tag(b"ls30")(input)?;
    parse_font(input, PrinterKind::Laser30)
}

/// Parse any printer font
pub fn parse_pset<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], PSet<'a>, E> {
    let (input, cc) = four_cc(input)?;
    match cc {
        FourCC::PS24 => parse_font(input, PrinterKind::Needle24),
        FourCC::PS09 => parse_font(input, PrinterKind::Needle9),
        FourCC::LS30 => parse_font(input, PrinterKind::Laser30),
        _ => {
            let e: ErrorKind = ErrorKind::Tag;
            Err(nom::Err::Error(E::from_error_kind(input, e)))
        }
    }
}
