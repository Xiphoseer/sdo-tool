# pdf-create

[![Crates.io](https://img.shields.io/crates/v/pdf-create.svg)](https://crates.io/crates/pdf-create)
[![Docs]( https://docs.rs/pdf-create/badge.svg)](https://docs.rs/pdf-create)

This is yet another PDF creation library, developed as part of the
[Signum!-Document-Toolbox](https://xiphoseer.github.io/sdo-tool). In
some respect, it's a prototype for missing parts of the [`pdf`] crate.

## Rationale

The main goal of this crate is the ability to use custom Type3 fonts to
create a document. The main design philosophy is the idea of storing all
data as rustic types and serializing them through a trait similar to
the `std::fmt` API.

As a PDF is fundamentally a list of objects that are linked using
their IDs and serializing some struct requires the referenced IDs
to be known, this crate has a `high`-level and a `low`-level component
that represent the document before and after assigning global IDs.

## Features

* Arbitrary content & glyph streams
* Custom `/Info` values
* Type3 fonts
* Outlines
* Page Labels
* Implementations of `Ascii85Decode` encoding and `PDFDocEncoding`

## Basic Usage

This example creates a single empty A4 page with no text:

```rust
use chrono::Local;
use pdf_create::{
    common::Point,
    common::{PdfString, Rectangle},
    high::{Handle, Page, Resources},
};

// Create a new handle
let mut doc = Handle::new();

// Set some metadata
doc.info.author = Some(PdfString::new("Xiphoseer"));
doc.info.creator = Some(PdfString::new("SIGNUM (c) 1986-93 F. Schmerbeck"));
doc.info.producer = Some(PdfString::new("Signum! Document Toolbox"));
doc.info.title = Some(PdfString::new("EMPTY.SDO"));
doc.info.mod_date = Some(Local::now());
doc.info.creation_date = Some(Local::now());

// Create a page
let page = Page {
    media_box: Rectangle {
        ll: Point { x: 0, y: 0 },
        ur: Point { x: 592, y: 842 },
    },
    resources: Resources::default(),
    contents: Vec::new(),
};

// Add the page to the document
doc.pages.push(page);

// Write the PDF to the console
let stdout = std::io::stdout();
let mut stdolock = stdout.lock();
doc.write(&mut stdolock)?;
```

## Alternatives

* If you are looking for a crate that can generate valid PDF files
  with arbitrary content, you should probably use [`lopdf`]
* If you are looking for a crate to save a combination of graphics
  and text as a PDF, you should probably use [`printpdf`],
  [`genpdf`] or [`pdf-canvas`]
* If you are looking for a crate that can load and render a PDF, you should
  probably use [`pdf`]

[`genpdf`]: https://crates.io/crates/genpdf
[`lopdf`]: https://crates.io/crates/lopdf
[`pdf`]: https://crates.io/crates/pdf
[`pdf-canvas`]: https://crates.io/crates/pdf-canvas
[`pdf-derive`]: https://crates.io/crates/pdf-derive
[`printpdf`]: https://crates.io/crates/printpdf