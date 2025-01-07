use pdf_create::chrono::Local;
use pdf_create::common::PdfString;
use pdf_create::encoding::{pdf_doc_encode, PDFDocEncodingError};
use pdf_create::high::Info;

/// Information to add into the PDF `/Info` dictionary
pub struct MetaInfo {
    /// Title
    pub title: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Subject
    pub subject: Option<String>,
}

pub fn prepare_info(info: &mut Info, meta: &MetaInfo) -> Result<(), PDFDocEncodingError> {
    if let Some(author) = &meta.author {
        let author = pdf_doc_encode(author)?;
        info.author = Some(PdfString::new(author));
    }
    if let Some(subject) = &meta.subject {
        let subject = pdf_doc_encode(subject)?;
        info.subject = Some(PdfString::new(subject));
    }
    if let Some(title) = &meta.title {
        let title = pdf_doc_encode(title)?;
        info.title = Some(PdfString::new(title));
    }
    let creator = pdf_doc_encode("SIGNUM © 1986-93 F. Schmerbeck")?;
    info.creator = Some(PdfString::new(creator));
    let producer = pdf_doc_encode("Signum! Document Toolbox")?;
    info.producer = Some(PdfString::new(producer));
    let now = Local::now();
    info.creation_date = Some(now);
    info.mod_date = Some(now);
    Ok(())
}
