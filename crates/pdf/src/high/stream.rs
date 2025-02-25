use std::borrow::Cow;

use crate::{common::StreamMetadata, low, lowering::DebugName, write::Serialize};

pub(crate) trait ToStream<'a> {
    type Stream: Serialize;

    fn to_stream(&'a self) -> Self::Stream;
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

    fn to_stream(&'a self) -> Self::Stream {
        low::Ascii85Stream {
            data: Cow::Borrowed(self.data.as_ref()),
            meta: self.meta,
        }
    }
}
