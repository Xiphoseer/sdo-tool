//! Errors relating to character sets

/// Glyph size error
#[derive(Debug)]
pub enum ChsetSizeError {
    /// A provided bitmap was not of the expected size
    UnexpectedBitmapSize {
        /// The expected size (calculated from width, height)
        expected: usize,
        /// The actual size
        actual: usize,
    },
}
