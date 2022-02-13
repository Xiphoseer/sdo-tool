#![warn(missing_docs)]
//! # pdf-create
//!
//! Library to create a PDF file with a rustic API
//!
//! ```
//! use chrono::Local;
//! use pdf_create::{
//!     common::Point,
//!     common::{PdfString, Rectangle},
//!     high::{Handle, Page, Resources},
//! };
//!
//! // Create a new handle
//! let mut doc = Handle::new();
//!
//! // Set some metadata
//! doc.info.author = Some(PdfString::new("Xiphoseer"));
//! doc.info.creator = Some(PdfString::new("SIGNUM (c) 1986-93 F. Schmerbeck"));
//! doc.info.producer = Some(PdfString::new("Signum! Document Toolbox"));
//! doc.info.title = Some(PdfString::new("EMPTY.SDO"));
//! doc.info.mod_date = Some(Local::now());
//! doc.info.creation_date = Some(Local::now());
//!
//! // Create a page
//! let page = Page {
//!     media_box: Rectangle {
//!         ll: Point { x: 0, y: 0 },
//!         ur: Point { x: 592, y: 842 },
//!     },
//!     resources: Resources::default(),
//!     contents: Vec::new(),
//! };
//!
//! // Add the page to the document
//! doc.pages.push(page);
//!
//! // Write the PDF to the console
//! let stdout = std::io::stdout();
//! let mut stdolock = stdout.lock();
//! doc.write(&mut stdolock).expect("Write to stdout");
//! ```
//!
//! Reference: <https://www.adobe.com/content/dam/acom/en/devnet/pdf/PDF32000_2008.pdf>

pub mod common;
pub mod encoding;
pub mod high;
pub mod low;
pub mod lowering;
pub mod util;
pub mod write;

#[doc(hidden)]
pub extern crate chrono;
