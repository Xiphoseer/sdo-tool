use std::fmt;

/// An error when parsing a CCITT encoded bi-level image
#[non_exhaustive]
#[derive(Debug)]
pub enum FaxError {}

impl std::error::Error for FaxError {}

impl fmt::Display for FaxError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}

/// Type alias for convenience
pub type FaxResult<T> = Result<T, FaxError>;
