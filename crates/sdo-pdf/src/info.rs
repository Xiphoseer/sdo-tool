use pdf_create::chrono::{DateTime, FixedOffset, Local, NaiveDateTime, TimeZone};
use pdf_create::common::{
    ColorSpace, ICCColorProfileMetadata, OutputIntent, OutputIntentSubtype, PdfString,
};
use pdf_create::encoding::PDFDocEncodingError;
use pdf_create::high::{Handle, ICCBasedColorProfile, Metadata};
use signum::docs::header::Header;

/// Information to add into the PDF `/Info` dictionary
#[derive(Debug, Clone, Default)]
pub struct MetaInfo {
    /// Title
    pub title: Option<String>,
    /// Author
    pub author: Vec<String>,
    /// Subject
    pub subject: Option<String>,

    /// Creation date of the document
    pub creation_date: Option<DateTime<FixedOffset>>,
    /// Date when the document was last updated
    pub mod_date: Option<DateTime<FixedOffset>>,
}

impl MetaInfo {
    /// Set the dates from the SDOC header
    pub fn with_dates(&mut self, header: &Header) {
        let ctime = NaiveDateTime::from(header.ctime);
        let mtime = NaiveDateTime::from(header.mtime);
        let tz = Local; // FIXME: timezone?
        self.creation_date = tz
            .from_local_datetime(&ctime)
            .single()
            .map(|d| d.fixed_offset());
        self.mod_date = tz
            .from_local_datetime(&mtime)
            .single()
            .map(|d| d.fixed_offset());
    }
}

/// Write PDF info data
pub fn prepare_info(info: &mut Metadata, meta: &MetaInfo) -> Result<(), PDFDocEncodingError> {
    info.author = meta.author.clone();
    if let Some(subject) = &meta.subject {
        info.subject = Some(subject.clone());
    }
    if let Some(title) = &meta.title {
        info.title = Some(title.clone());
    }
    info.creator = Some("SIGNUM © 1986-93 F. Schmerbeck".to_owned());
    info.producer = "Signum! Document Toolbox".to_owned();
    if let Some(creation_date) = meta.creation_date {
        info.creation_date = creation_date;
    }
    if let Some(mod_date) = meta.mod_date {
        info.modify_date = mod_date;
    }
    Ok(())
}

/// Add a simple output intent for PDF/A
///
/// This is not yet properly implemented
pub fn prepare_pdfa_output_intent(hnd: &mut Handle) -> crate::Result<()> {
    hnd.output_intents.push(PdfAOutputIntent::default_grey());
    Ok(())
}

struct PdfAOutputIntent;

impl PdfAOutputIntent {
    /// Return a minimal grayscale sRGB output intent
    fn default_grey() -> OutputIntent<ICCBasedColorProfile<'static>> {
        OutputIntent {
            subtype: OutputIntentSubtype::GTS_PDFA1,
            output_condition_identifier: PdfString::new(b"sGry"),
            output_condition: None,
            registry_name: None,
            info: Some(PdfString::new(
                b"Compact-ICC-Profiles ICC v4 Grayscale Parametric Curve",
            )),
            dest_output_profile: Some(ICC_SGREY_V4),
        }
    }
}

// https://github.com/saucecontrol/Compact-ICC-Profiles
const ICC_SGREY_V4: ICCBasedColorProfile<'static> = ICCBasedColorProfile {
    stream: include_bytes!("../res/sGrey-v4.icc"),
    meta: ICCColorProfileMetadata {
        alternate: Some(ColorSpace::DeviceGray),
        num_components: 1,
    },
};
