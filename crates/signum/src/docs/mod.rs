//! # The Signum! document
//!
//! This module contains the datastructures and parsers for reading SDO files.

use hcim::{ImageEntry, ImageSite};
use log::info;
use nom::{
    combinator::map,
    error::{ContextError, ParseError},
    number::complete::{be_u16, be_u32, le_u32},
    Finish, IResult,
};
use tebu::PageText;

use crate::{
    chsets::cache::DocumentFontCacheInfo,
    raster::Page,
    util::{Bytes16, Bytes32, FourCC},
};
use fmt::Debug;
use std::{borrow::Cow, collections::BTreeMap, fmt};

use self::{
    container::SDocContainer, cset::CSet, hcim::Hcim, header::Header, pbuf::PBuf, sysp::SysP,
    tebu::TeBu,
};

pub mod container;
pub mod cset;
pub mod hcim;
pub mod header;
pub mod pbuf;
pub mod sysp;
pub mod tebu;

mod error {
    use nom::error::ErrorKind;

    use crate::util::FourCC;

    /// A generic error
    #[derive(Debug, Copy, Clone)]
    pub enum Error {
        /// Parser Error
        Nom {
            /// The container tag where the problem happened
            chunk_tag: FourCC,
            /// The type of error
            code: ErrorKind,
            /// The offset of the error
            offset: usize,
        },
        /// Missing a tag
        MissingTag(FourCC),
    }
}

pub use error::Error;

#[derive(Debug)]
/// FIXME: Implement this to load a full document
pub struct SDoc<'a> {
    /// The header of the document
    pub header: Header<'a>,
    /// Character sets
    pub cset: CSet<'a>,
    /// System Paramters
    pub sysp: SysP,
    /// Page Buffer
    pub pbuf: PBuf<'a>,
    /// Text Buffer
    pub tebu: TeBu,
    /// Hardcopy Images
    pub hcim: Option<Hcim<'a>>,
    /// Other unparsed chunks
    pub other: BTreeMap<FourCC, Cow<'a, [u8]>>,
}

type NomErr<'a> = nom::error::Error<&'a [u8]>;

impl<'a> SDoc<'a> {
    /// Turn this into an owned variant
    ///
    /// Allocates all previously borrowed buffers
    pub fn into_owned(self) -> SDoc<'static> {
        let SDoc {
            header,
            cset,
            sysp,
            pbuf,
            tebu,
            hcim,
            other,
        } = self;
        let header = header.into_owned();
        let cset = cset.into_owned();
        let pbuf = pbuf.into_owned();
        let hcim = hcim.map(Hcim::into_owned);
        let other = other
            .into_iter()
            .map(|(key, value)| (key, Cow::Owned(value.into_owned())))
            .collect();
        SDoc {
            header,
            cset,
            sysp,
            pbuf,
            tebu,
            hcim,
            other,
        }
    }

    /// Get a slice of all image sites
    pub fn image_sites(&self) -> &[ImageSite] {
        self.hcim
            .as_ref()
            .map(|hcim| hcim.sites.as_slice())
            .unwrap_or(&[])
    }

    /// Unpack a document from a container
    pub fn unpack(container: SDocContainer<'a>) -> Result<Self, Error> {
        let mut header = None;
        let mut cset = None;
        let mut sysp = None;
        let mut pbuf = None;
        let mut tebu = None;
        let mut hcim = None;
        let mut other = BTreeMap::new();
        for chunk in container.chunks {
            info!("Parsing {}", chunk.tag);
            match chunk.tag {
                FourCC::_0001 => {
                    header = Some(Header::unpack(chunk)?);
                }
                FourCC::_CSET => {
                    cset = Some(CSet::unpack(chunk)?);
                }
                SysP::TAG => {
                    sysp = Some(SysP::unpack(chunk)?);
                }
                PBuf::TAG => {
                    pbuf = Some(PBuf::unpack(chunk)?);
                }
                TeBu::TAG => {
                    tebu = Some(TeBu::unpack(chunk)?);
                }
                Hcim::TAG => {
                    hcim = Some(Hcim::unpack(chunk)?);
                }
                _ => {
                    other.insert(chunk.tag, Cow::Borrowed(chunk.buf.0));
                }
            }
        }

        Ok(Self {
            header: header.ok_or(Error::MissingTag(Header::TAG))?,
            cset: cset.ok_or(Error::MissingTag(CSet::TAG))?,
            sysp: sysp.ok_or(Error::MissingTag(SysP::TAG))?,
            pbuf: pbuf.ok_or(Error::MissingTag(PBuf::TAG))?,
            tebu: tebu.ok_or(Error::MissingTag(TeBu::TAG))?,
            hcim,
            other,
        })
    }
}

/// Take the next 16 bytes
pub fn bytes16<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Bytes16, E> {
    map(be_u16, Bytes16)(input)
}

/// Take the next 32 bytes
pub fn bytes32<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Bytes32, E> {
    map(be_u32, Bytes32)(input)
}

/// Parse a four character code
pub fn four_cc<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], FourCC, E> {
    map(map(le_u32, u32::to_le_bytes), FourCC)(input)
}

/// A chunk within the document
pub trait Chunk<'a>: Sized {
    /// The tag for this chunk
    const TAG: FourCC;

    /// The [`nom`] parser for this chunk
    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]> + ContextError<&'a [u8]>;

    /// Unpack a chunk into its typed version
    fn unpack(chunk: container::Chunk<'a>) -> Result<Self, Error> {
        let input = &chunk.buf.0;
        let chunk_tag = chunk.tag;
        let chunk_len = chunk.buf.0.len();
        let map_err = move |e: nom::error::Error<&'a [u8]>| Error::Nom {
            chunk_tag,
            code: e.code,
            offset: chunk_len - e.input.len(),
        };
        let (_, head) = Self::parse::<NomErr<'a>>(input).finish().map_err(map_err)?;
        Ok(head)
    }
}

/// Information commonly used in preparing document output
pub struct DocumentInfo {
    /// Information on how to locate the font information in a [crate::chsets::cache::ChsetCache]
    pub fonts: DocumentFontCacheInfo,
    /// Decoded images embedded in the document
    images: Vec<ImageEntry>,
}

impl DocumentInfo {
    /// Create a new instance
    pub fn new(fonts: DocumentFontCacheInfo, images: Vec<ImageEntry>) -> Self {
        Self { fonts, images }
    }

    /// Get the image associated with the given index
    pub fn image_at(&self, img: u16) -> &Page {
        &self.images[img as usize].image
    }

    /// Iterator over all images
    pub fn images(&self) -> impl Iterator<Item = &ImageEntry> {
        self.images.iter()
    }
}

/// Common adjustments to the graphics state for rendering
pub struct Overrides {
    /// Horizontal offset for rendering
    pub xoffset: i32,
    /// Vertical offset for rendering
    pub yoffset: i32,
}

/// Trait for providing document context to a generator
pub trait GenerationContext {
    /// Get a slice of all image sites (from hcim)
    fn image_sites(&self) -> &[ImageSite];

    /// Get system parameters
    fn sysp(&self) -> &SysP;

    /// Get the document information
    fn document_info(&self) -> &DocumentInfo;

    /// Get the text pages
    fn text_pages(&self) -> &[PageText];

    /// Get a specific page buffer page
    fn page_at(&self, index: usize) -> Option<&pbuf::Page>;

    /// Get the document font-cache info
    fn fonts(&self) -> &DocumentFontCacheInfo {
        &self.document_info().fonts
    }
}
