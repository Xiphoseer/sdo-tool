#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
//! # File formats from *Signum!*
//!
//! This crate is an implementation of the document file format (`*.SDO`)
//! and related formats, that were used by the word processor [Signum!]
//! published in 1986 by [Application Systems Heidelberg][ASH] (Germany).
//!
//! At the moment, only reading the files is supported.
//!
//! [Signum!]: https://de.wikipedia.org/wiki/Signum_(Textverarbeitungsprogramm)
//! [ASH]: https://application-systems.de

pub mod chsets;
pub mod docs;
pub mod images;
pub mod raster;
pub mod util;

#[cfg(feature = "image")]
#[doc(hidden)]
pub use image;
#[doc(hidden)]
pub use nom;
#[doc(hidden)]
pub use smallvec;
