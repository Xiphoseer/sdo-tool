#![warn(missing_docs)]
//! CCITT fax encodings

mod ascii_art;
pub mod bits;
mod color;
pub mod g42d;
mod store;

pub use ascii_art::ascii_art;
pub(crate) use ascii_art::ASCII;
pub use color::Color;
pub use store::{ColorLine, Store};
