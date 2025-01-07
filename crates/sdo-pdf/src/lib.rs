pub mod cmap;
pub mod font;
mod image;
mod info;
pub mod sdoc;

use core::fmt;

pub use info::{prepare_info, MetaInfo};

#[derive(Debug)]
pub enum Error {
    /// eyre!("Missing font #{}: {:?}", csu, font_name)
    MissingFont(usize, String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingFont(csu, font_name) => {
                write!(f, "Missing font #{}: {:?}", csu, font_name)
            }
        }
    }
}

impl std::error::Error for Error {}
