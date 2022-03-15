//! ## XMP generation facilities

use guid_create::GUID;
use std::{collections::BTreeMap, fmt, io};

/// Implemented for structures that can hold XMP description data
pub trait XmpDescription {
    /// Namespace of this description
    const NAMESPACE_URL: &'static str;
    /// Preferred namespace identifier
    const NAMESPACE_KEY: &'static str;

    /// Function to write the XML stream
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<()>;
}

/// Language Tag
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum Lang {
    /// **x-default**
    Default,
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default => write!(f, "x-default"),
        }
    }
}

/// PDF Metadata
pub struct Pdf {
    /// Program that produced the PDF (user of this library)
    pub producer: String,
}

impl XmpDescription for Pdf {
    const NAMESPACE_URL: &'static str = "http://ns.adobe.com/pdf/1.3/";
    const NAMESPACE_KEY: &'static str = "pdf";

    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        writeln!(w, "    <pdf:Producer>{}</pdf:Producer>", self.producer)?;
        Ok(())
    }
}

/// Dublin core metadata
pub struct DublinCore {
    /// Title in different languages
    pub title: BTreeMap<Lang, String>,
    /// MIME type (must be `application/pdf`)
    pub format: &'static str,
    /// Ordered List of creators
    pub creator: Vec<String>,
    /// Unordered set of publishers
    pub publisher: Vec<String>,
}

impl XmpDescription for DublinCore {
    const NAMESPACE_URL: &'static str = "http://purl.org/dc/elements/1.1/";
    const NAMESPACE_KEY: &'static str = "dc";

    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        writeln!(w, "   <dc:format>{}</dc:format>", self.format)?;

        if !self.title.is_empty() {
            write!(w, "   <dc:title><rdf:Alt>")?;
            for (k, v) in &self.title {
                write!(w, "<rdf:li xml:lang=\"{}\">{}</rdf:li>", k, v)?;
            }
            writeln!(w, "</rdf:Alt></dc:title>")?;
        }

        if !self.creator.is_empty() {
            write!(w, "   <dc:creator><rdf:Seq>")?;
            for creator in &self.creator {
                write!(w, "<rdf:li>{}</rdf:li>", creator)?;
            }
            writeln!(w, "</rdf:Seq></dc:creator>")?;
        }

        if !self.publisher.is_empty() {
            write!(w, "   <dc:publisher><rdf:Bag>")?;
            for publisher in &self.publisher {
                write!(w, "<rdf:li>{}</rdf:li>", publisher)?;
            }
            write!(w, "   </rdf:Bag></dc:publisher>")?;
        }
        Ok(())
    }
}

/// PDF/A Identification
pub struct PdfAId {
    /// Specifcation Part (1/2/3)
    pub part: u8,
    /// Conformance Variant (A/B/U)
    pub conformance: char,
}

impl XmpDescription for PdfAId {
    const NAMESPACE_URL: &'static str = "http://www.aiim.org/pdfa/ns/id/";
    const NAMESPACE_KEY: &'static str = "pdfaid";

    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        writeln!(w, "    <pdfaid:part>{}</pdfaid:part>", self.part)?;
        writeln!(
            w,
            "    <pdfaid:conformance>{}</pdfaid:conformance>",
            self.conformance
        )?;
        Ok(())
    }
}

/// Adobe XMP Basic namespace
pub struct XmpBasic {
    /// Tool used to create the document
    pub creator_tool: String,
    /// Time at last modification
    pub modify_date: chrono::DateTime<chrono::Local>,
    /// Time at creation
    pub create_date: chrono::DateTime<chrono::Local>,
    /// Time at metadata creation
    pub metadata_date: chrono::DateTime<chrono::Local>,
}

impl XmpDescription for XmpBasic {
    const NAMESPACE_URL: &'static str = "http://ns.adobe.com/xap/1.0/";
    const NAMESPACE_KEY: &'static str = "xmp";

    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        writeln!(
            w,
            "    <xmp:CreatorTool>{}</xmp:CreatorTool>",
            self.creator_tool
        )?;
        writeln!(
            w,
            "    <xmp:ModifyDate>{}</xmp:ModifyDate>",
            self.modify_date.format("%+")
        )?;
        writeln!(
            w,
            "    <xmp:CreateDate>{}</xmp:CreateDate>",
            self.create_date.format("%+")
        )?;
        writeln!(
            w,
            "    <xmp:MetadataDate>{}</xmp:MetadataDate>",
            self.metadata_date.format("%+")
        )?;
        Ok(())
    }
}

/// Adobe XMP Media Management
pub struct XmpMM {
    /// Unique identifier for this document
    pub document_id: GUID,
    /// Unqiue identifier for the latest modification
    pub instance_id: GUID,
}

impl XmpDescription for XmpMM {
    const NAMESPACE_URL: &'static str = "http://ns.adobe.com/xap/1.0/mm/";
    const NAMESPACE_KEY: &'static str = "xmpMM";

    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        writeln!(
            w,
            "    <xmpMM:DocumentID>uuid:{}</xmpMM:DocumentID>",
            self.document_id
        )?;
        writeln!(
            w,
            "    <xmpMM:InstanceID>uuid:{}</xmpMM:InstanceID>",
            self.instance_id
        )?;
        Ok(())
    }
}

const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const XMPTK: &str = "Adobe XMP Core 4.0-c316 44.253921, Sun Oct 01 2006 17:14:39";
const META_NS: &str = "adobe:ns:meta/";

/// Writer for XMP metadata
pub struct XmpWriter<W>(W);

impl<W: io::Write> XmpWriter<W> {
    /// Create a new instance
    pub fn new(mut w: W) -> io::Result<Self> {
        writeln!(
            w,
            "<?xpacket begin='\u{FEFF}' id='W5M0MpCehiHzreSzNTczkc9d' ?>"
        )?;
        writeln!(
            w,
            "<x:xmpmeta xmlns:x=\"{}\" x:xmptk=\"{}\">",
            META_NS, XMPTK
        )?;
        writeln!(w, " <rdf:RDF xmlns:rdf=\"{}\">", RDF_NS)?;
        Ok(Self(w))
    }

    /// Add a description
    pub fn add_description<X: XmpDescription>(&mut self, desc: &X) -> io::Result<()> {
        let w = &mut self.0;
        writeln!(
            w,
            "   <rdf:Description rdf:about=\"\" xmlns:{}=\"{}\">",
            X::NAMESPACE_KEY,
            X::NAMESPACE_URL
        )?;
        desc.write(w)?;
        writeln!(w, "   </rdf:Description>")?;
        Ok(())
    }

    /// Finish the data
    pub fn finish(self) -> io::Result<W> {
        let mut w = self.0;
        writeln!(w, " </rdf:RDF>")?;
        writeln!(w, "</x:xmpmeta>")?;
        writeln!(w, "<?xpacket end='w'?>")?;
        Ok(w)
    }
}
