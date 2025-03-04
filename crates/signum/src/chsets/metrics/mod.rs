//! # Font Metrics
//!
//! This module describes the font metrics of *Signum fonts* in terms
//! of standard typographic units and terms.
//!
//! - A (DTP) **point** is 1/72 of an inch
//! - The *em square* is the bbox in which any glyph must fit
//! - The *font size* defines the height of the *em square* in points
//! - A *font unit* is the subdivision of the *em square*, usually 1/1000 or 1/1024

use super::printer::PrinterKind;

pub mod widths;

/// Number of DTP [points] in one inch (72)
///
/// [points]: https://en.wikipedia.org/wiki/Point_(typography)
pub const POINTS_PER_INCH: u32 = 72;

/// Number of font-units in one *em square* (1000)
pub const UNITS_PER_EM: u32 = 1000;

/// Size of the em square in font-units
///
/// Equal to `(UNITS_PER_EM, UNITS_PER_EM)` i.e. `(1000, 1000)`
pub const EM_SQUARE: (u32, u32) = (UNITS_PER_EM, UNITS_PER_EM);

/// The number of font units per inch for a 1pt font size
pub const FONTUNITS_PER_INCH_UNSCALED: u32 = POINTS_PER_INCH * UNITS_PER_EM;

/// The "default" assumed font size for a signum font (10pt)
///
/// This is selected, such that a signum printer glyph roughly fits into
/// a (1000 unit) em square given [UNITS_PER_EM] and the known printer
/// resolution.
pub const DEFAULT_FONT_SIZE: u32 = 10;

/// This struct describes the metrics of a font for a particular rendering size.
#[non_exhaustive]
pub struct FontMetrics {
    /// font-units per pixel of a signum font
    fontunits_per_pixel_x: u32,
    /// font-units per pixel of a signum font
    fontunits_per_pixel_y: u32,
}

impl FontMetrics {
    /// Create a new font metric
    pub fn new(pk: PrinterKind, font_size: u32) -> Self {
        let pixels_per_inch = pk.resolution();

        let fontunits_per_pixel_x = FONTUNITS_PER_INCH_UNSCALED / font_size / pixels_per_inch.x;
        let fontunits_per_pixel_y = FONTUNITS_PER_INCH_UNSCALED / font_size / pixels_per_inch.y;
        Self {
            fontunits_per_pixel_x,
            fontunits_per_pixel_y,
        }
    }

    /// Convert pixel lengths (in signum font resolution)
    /// to *unscaled* font units.
    pub fn pixels_to_fontunits(&self, (x, y): (u32, u32)) -> (u32, u32) {
        (
            x * self.fontunits_per_pixel_x,
            y * self.fontunits_per_pixel_y,
        )
    }

    /// Convert font units to pixels (in signum font resolution)
    ///
    /// ```
    /// # use signum::chsets::{printer::PrinterKind, metrics::FontMetrics};
    /// let fm_p24 = FontMetrics::from(PrinterKind::Needle24);
    /// assert_eq!((50,50), fm_p24.fontunits_to_pixels((1000, 1000)));
    /// let fm_p9 = FontMetrics::from(PrinterKind::Needle9);
    /// assert_eq!((33,30), fm_p9.fontunits_to_pixels((1000, 1000)));
    /// let fm_l30 = FontMetrics::from(PrinterKind::Laser30);
    /// assert_eq!((41,41), fm_l30.fontunits_to_pixels((1000, 1000)));
    /// ```
    pub fn fontunits_to_pixels(&self, (x, y): (u32, u32)) -> (u32, u32) {
        (
            x / self.fontunits_per_pixel_x,
            y / self.fontunits_per_pixel_y,
        )
    }

    /// Return the size of the em square in pixels
    ///
    /// Returns the bigger value of [Self::fontunits_to_pixels] called with `(1000, 1000)` i.e. [`EM_SQUARE`]
    ///
    /// ```
    /// # use signum::chsets::{printer::PrinterKind, metrics::FontMetrics};
    /// let fm_p24 = FontMetrics::from(PrinterKind::Needle24);
    /// assert_eq!(50, fm_p24.em_square_pixels());
    /// let fm_p9 = FontMetrics::from(PrinterKind::Needle9);
    /// assert_eq!(33, fm_p9.em_square_pixels());
    /// let fm_l30 = FontMetrics::from(PrinterKind::Laser30);
    /// assert_eq!(41, fm_l30.em_square_pixels());
    /// ```
    pub fn em_square_pixels(&self) -> u32 {
        let (x, y) = self.fontunits_to_pixels(EM_SQUARE);
        x.max(y)
    }

    /// Return the font-units per pixel (of signum font)
    ///
    /// ```
    /// # use signum::chsets::{printer::PrinterKind, metrics::FontMetrics};
    /// let fm = FontMetrics::from(PrinterKind::Needle24);
    /// assert_eq!((20, 20), fm.fontunits_per_pixel());
    /// ```
    pub fn fontunits_per_pixel(&self) -> (u32, u32) {
        (self.fontunits_per_pixel_x, self.fontunits_per_pixel_y)
    }
}

impl From<PrinterKind> for FontMetrics {
    /// Get the font metrics for the given printer, assuming the specified font size
    fn from(value: PrinterKind) -> Self {
        Self::new(value, DEFAULT_FONT_SIZE)
    }
}

/// A font bounding box
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BBox {
    /// lower-left x
    pub ll_x: usize,
    /// lower-left y
    pub ll_y: i32,
    /// upper-right x
    pub ur_x: usize,
    /// upper-right y
    pub ur_y: i32,
}

impl BBox {
    /// Get the width of the box
    pub fn width(&self) -> usize {
        self.ur_x - self.ll_x
    }

    /// Get the height of the box
    pub fn height(&self) -> i32 {
        self.ur_y - self.ll_y
    }
}
