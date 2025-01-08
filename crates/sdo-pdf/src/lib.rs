pub mod cmap;
pub mod font;
mod image;
mod info;
pub mod sdoc;

use std::{fmt, io};

use font::prepare_pdf_fonts;
pub use info::{prepare_info, prepare_pdfa_output_intent, MetaInfo};
use pdf_create::{encoding::PDFDocEncodingError, high::Handle};
use sdoc::generate_pdf_pages;
use signum::{
    chsets::{cache::ChsetCache, printer::PrinterKind},
    docs::{GenerationContext, Overrides},
};

#[derive(Debug)]
pub enum Error {
    /// Missing font #{}: {:?}
    MissingFont(usize, String),
    /// PDF encoding error
    Encoding(PDFDocEncodingError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingFont(csu, font_name) => {
                write!(f, "Missing font #{}: {:?}", csu, font_name)
            }
            Error::Encoding(pdfdoc_encoding_error) => {
                <PDFDocEncodingError as fmt::Display>::fmt(pdfdoc_encoding_error, f)
            }
        }
    }
}

impl From<PDFDocEncodingError> for Error {
    fn from(value: PDFDocEncodingError) -> Self {
        Self::Encoding(value)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Pdf<'a> {
    hnd: Handle<'a>,
}

impl<'a> Pdf<'a> {
    pub fn write<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        self.hnd.write(w)
    }

    pub fn from_raw(hnd: Handle<'a>) -> Self {
        Self { hnd }
    }
}

pub fn generate_pdf<'f, GC: GenerationContext>(
    fc: &'f ChsetCache,
    pk: PrinterKind,
    meta: &MetaInfo,
    overrides: &Overrides,
    gc: &GC,
) -> crate::Result<Pdf<'f>> {
    let mut hnd = Handle::new();
    prepare_info(&mut hnd.info, meta)?;
    prepare_pdfa_output_intent(&mut hnd)?;
    let font_info = prepare_pdf_fonts(&mut hnd.res.fonts, gc, fc, pk);
    generate_pdf_pages(gc, &mut hnd, overrides, &font_info)?;
    Ok(Pdf { hnd })
}
