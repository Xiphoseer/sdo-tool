//! # Signum! 3/4 Documents
//!
//! Signum! 3/4 uses a completely different format from 1/2.

use nom::{
    bytes::complete::take,
    combinator::cond,
    error::{ContextError, ParseError},
    IResult,
};

pub use flptrs::FilePointers;
pub use foused::{FontUsed, FontsUsed};
pub use header::Header;
pub use kapit::ChapterHeader;
pub use stream::Stream;

use crate::util::V3Chunk;

mod flptrs;
mod foused;
mod header;
mod kapit;
mod stream;

/// Tag for a v3 document
pub const TAG_SDOC3: &[u8; 12] = b"\0\0sdoc  03\0\0";

/// Document root
#[allow(dead_code)]
#[derive(Debug)]
pub struct SDocV3<'a> {
    /// Header buffer
    header: Header<'a>,
    /// The file pointers
    file_pointers: FilePointers,
    /// The fonts in the file
    fonts: FontsUsed<'a>,
    /// The chapters in the documents
    chapters: Vec<Chapter<'a>>,
}

impl<'a> SDocV3<'a> {
    /// Get the *file pointers* `flptrs01` chunk
    pub fn sdoc03(&self) -> &Header<'a> {
        &self.header
    }

    /// Get the *file pointers* `flptrs01` chunk
    pub fn flptrs01(&self) -> &FilePointers {
        &self.file_pointers
    }

    /// Get the *fonts used* `foused01` chunk
    pub fn foused01(&self) -> &FontsUsed<'a> {
        &self.fonts
    }

    /// Get the *chapters* `kapit 01` + `stream01` chunks
    pub fn chapters(&self) -> &[Chapter<'a>] {
        &self.chapters
    }
}

/// A *chapter* (`kapit 01` and `stream01` chunks)
#[derive(Debug)]
#[allow(dead_code)]
pub struct Chapter<'a> {
    header: ChapterHeader<'a>,
    main: Stream<'a>,
    head_foot: Stream<'a>,
    s3: Option<Stream<'a>>,
    s4: Option<Stream<'a>>,
}

impl<'a> Chapter<'a> {
    /// Get the header of a chapter
    pub fn header(&self) -> &ChapterHeader<'a> {
        &self.header
    }

    /// Return the main `stream01`
    pub fn main(&self) -> &Stream<'a> {
        &self.main
    }

    /// Return the main `stream01`
    pub fn header_footer(&self) -> &Stream<'a> {
        &self.head_foot
    }

    /// Return the 3rd `stream01`
    pub fn stream3(&self) -> Option<&Stream<'a>> {
        self.s3.as_ref()
    }

    /// Return the 4th `stream01`, if present
    pub fn stream4(&self) -> Option<&Stream<'a>> {
        self.s4.as_ref()
    }
}

/// Parse a Signum! document
pub fn parse_sdoc_v3<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], SDocV3<'a>, E>
where
    E: ParseError<&'a [u8]>,
    E: ContextError<&'a [u8]>,
{
    let data = input;
    let (input, header) = Header::parse_chunk(input)?;
    let (input, file_pointers) = FilePointers::parse_chunk(input)?;

    let (_, fonts) = {
        let (i_foused01, _) = take(file_pointers.ofs_foused01)(data)?;
        FontsUsed::parse_chunk(i_foused01)
    }?;

    let mut chapters = Vec::new();
    for &ofs_kapit in &file_pointers.ofs_chapters {
        let (input, _) = take(ofs_kapit)(data)?;
        let (input, kapit) = ChapterHeader::parse_chunk(input)?;
        let (input, main) = Stream::parse_chunk(input)?;
        let (input, head_foot) = Stream::parse_chunk(input)?;
        let (input, s3) = cond(kapit.v9 >= 0, Stream::parse_chunk)(input)?;
        let (input, s4) = cond(kapit.v10 >= 0, Stream::parse_chunk)(input)?;
        let _ = input;
        chapters.push(Chapter {
            header: kapit,
            main,
            head_foot,
            s3,
            s4,
        })
    }

    Ok((
        input,
        SDocV3 {
            header,
            file_pointers,
            fonts,
            chapters,
        },
    ))
}
