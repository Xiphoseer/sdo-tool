use std::{borrow::Cow, convert::Infallible};

use crate::{
    common::{ImageMetadata, StreamMetadata},
    low,
    lowering::DebugName,
};

use super::stream::ToStream;

/// An embedded object resource
#[derive(Debug)]
pub enum XObject {
    /// An image
    Image(Image),
}

#[derive(Debug)]
/// An Image resource
pub struct Image {
    /// The metadata for this image
    pub meta: ImageMetadata,
    /// The data for the image
    pub data: Vec<u8>,
}

impl From<Image> for XObject {
    fn from(value: Image) -> Self {
        XObject::Image(value)
    }
}

impl DebugName for XObject {
    fn debug_name() -> &'static str {
        "XObject"
    }
}

impl<'a> ToStream<'a> for XObject {
    type Stream = low::Ascii85Stream<'a>;
    type Error = Infallible;

    fn to_stream(&'a self) -> Result<Self::Stream, Self::Error> {
        match self {
            Self::Image(i) => Ok(low::Ascii85Stream {
                data: Cow::Borrowed(&i.data),
                meta: StreamMetadata::Image(i.meta),
            }),
        }
    }
}
