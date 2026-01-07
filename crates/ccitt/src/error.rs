/// An error when parsing a CCITT encoded bi-level image
#[non_exhaustive]
#[derive(Debug)]
pub enum FaxError {}

/// Type alias for convenience
pub type FaxResult<T> = Result<T, FaxError>;
