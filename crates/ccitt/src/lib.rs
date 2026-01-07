#![warn(missing_docs)]
//! # CCITT (ITU-T) fax encodings
//!
//! This crate implements encoders and decoders for the bi-level image
//! compression as defined by the T.4 and T.6 Recommendations. These can
//! commonly be found embedded in PDF or TIFF files or produced by fax
//! software.
//!
//! It was created as part of [*Signum! Document Toolbox*][sdo-tool],
//! you might want to use the [`fax`] crate instead.
//!
//! [sdo-tool]: https://sdo.dseiler.eu
//! [`fax`]: https://crates.io/crates/fax

mod ascii_art;
pub mod bits;
mod color;
mod error;
pub mod g3;
pub mod g42d;
mod image;
mod store;
pub(crate) mod terminals;

pub(crate) use ascii_art::ASCII;
pub use ascii_art::{ascii_art, pbm_to_io_writer};
pub use color::Color;
pub use error::{FaxError, FaxResult};
pub use image::FaxImage;
pub use store::{ColorLine, Store};
