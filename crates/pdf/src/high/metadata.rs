use std::io;

use chrono::{DateTime, FixedOffset, Local};
use uuid::Uuid;

use crate::{
    common::{PdfString, Trapped},
    write::{Formatter, Serialize},
};

/// The Metadata/Info
#[derive(Debug, Default)]
pub struct Metadata {
    /// The title
    pub title: Option<String>,
    /// The author
    pub author: Vec<String>,
    /// The subject
    pub subject: Option<String>,
    /// A list of keywords
    pub keywords: Vec<String>,
    /// The program used to create the source
    pub creator: Option<String>,
    /// The program used to create the source
    pub publisher: Vec<String>,
    /// The program that produced the file (this library)
    pub producer: String,

    /// Date-Time at which the document was created
    pub creation_date: DateTime<FixedOffset>,
    /// Date-Time at which the document was modified
    pub modify_date: DateTime<FixedOffset>,

    /// XMP Media Management: Document ID
    pub document_id: Uuid,
    /// XMP Media Management: Instance ID
    pub instance_id: Uuid,
}

impl Metadata {
    /// Create a new instance with the current date-time for created & modified
    pub fn new() -> Self {
        let now = Local::now().fixed_offset();
        Self {
            creation_date: now,
            modify_date: now,
            document_id: Uuid::new_v4(),
            instance_id: Uuid::new_v4(),
            ..Default::default()
        }
    }

    /// Check whether the info contains any meaningful data
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.author.is_empty()
            && self.subject.is_none()
            && self.keywords.is_empty()
            && self.publisher.is_empty()
            && self.creator.is_none()
            && self.producer.is_empty()
    }
}

/// The Metadata/Info
#[derive(Debug, Default)]
pub struct Info {
    /// The title
    pub title: Option<PdfString>,
    /// The author
    pub author: Option<PdfString>,
    /// The subject
    pub subject: Option<PdfString>,
    /// A list of keywords
    pub keywords: Option<PdfString>,
    /// The program used to create the source
    pub creator: Option<PdfString>,
    /// The program that produced the file (this library)
    pub producer: Option<PdfString>,

    /// The date of creation
    pub creation_date: Option<DateTime<FixedOffset>>,
    /// The date of the last modification
    pub mod_date: Option<DateTime<FixedOffset>>,

    /// Whether the PDF is *trapped*
    pub trapped: Option<Trapped>,
}

impl Serialize for Info {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        let mut dict = f.pdf_dict();
        if let Some(title) = &self.title {
            dict.field("Title", title)?;
        }
        if let Some(author) = &self.author {
            dict.field("Author", author)?;
        }
        if let Some(subject) = &self.subject {
            dict.field("Subject", subject)?;
        }
        if let Some(keywords) = &self.keywords {
            dict.field("Keywords", keywords)?;
        }
        if let Some(creator) = &self.creator {
            dict.field("Creator", creator)?;
        }
        if let Some(producer) = &self.producer {
            dict.field("Producer", producer)?;
        }

        if let Some(creation_date) = &self.creation_date {
            dict.field("CreationDate", creation_date)?;
        }
        if let Some(mod_date) = &self.mod_date {
            dict.field("ModDate", mod_date)?;
        }
        if let Some(trapped) = &self.trapped {
            dict.field("Trapped", trapped)?;
        }
        dict.finish()?;
        Ok(())
    }
}
