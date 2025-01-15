use pdf_create::chrono::Local;
use pdf_create::common::{OutputIntent, OutputIntentSubtype, PdfString};
use pdf_create::encoding::{pdf_doc_encode, PDFDocEncodingError};
use pdf_create::high::{Handle, Info};

/// Information to add into the PDF `/Info` dictionary
#[derive(Debug, Clone, Default)]
pub struct MetaInfo {
    /// Title
    pub title: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Subject
    pub subject: Option<String>,
}

/// Write PDF info data
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

/// Add a simple output intend for PDF/A
///
/// This is not yet properly implemented
pub fn prepare_pdfa_output_intent(hnd: &mut Handle) -> crate::Result<()> {
    // Output intents
    hnd.output_intents.push(OutputIntent {
        subtype: OutputIntentSubtype::GTS_PDFA1,
        output_condition: None,
        output_condition_identifier: PdfString::new("FOO"),
        registry_name: None,
        info: None,
    });
    Ok(())
}
