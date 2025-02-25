use std::{borrow::Cow, convert::Infallible, fmt};

use crate::{common::StreamMetadata, low, lowering::DebugName, write::Serialize};

pub(crate) trait ToStream<'a> {
    type Stream: Serialize;
    type Error: fmt::Debug;

    fn to_stream(&'a self) -> Result<Self::Stream, Self::Error>;
}

#[derive(Debug, Clone)]
/// A text stream in the PDF
pub struct Ascii85Stream<'a> {
    /// The data of this stream
    pub data: Cow<'a, [u8]>,
    /// The metadata for this stream
    pub meta: StreamMetadata,
}

impl DebugName for Ascii85Stream<'_> {
    fn debug_name() -> &'static str {
        "Ascii85Stream"
    }
}

impl<'a> ToStream<'a> for Ascii85Stream<'a> {
    type Stream = low::Ascii85Stream<'a>;
    type Error = Infallible;

    fn to_stream(&'a self) -> Result<Self::Stream, Infallible> {
        Ok(low::Ascii85Stream {
            data: Cow::Borrowed(self.data.as_ref()),
            meta: self.meta,
        })
    }
}
