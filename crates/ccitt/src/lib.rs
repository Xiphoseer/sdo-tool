#![warn(missing_docs)]
//! CCITT fax encodings

pub mod bits;
mod color;
pub mod g42d;
mod store;

pub use color::Color;
pub use store::{ColorLine, Store};
