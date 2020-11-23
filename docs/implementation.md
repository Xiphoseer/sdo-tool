# Implementation

The SDO-Toolbox is a CLI Application and collection of libraries written
in the [Rust Programming Language](https://rust-lang.org).

## PDF-Create

The main user-facing feature of the toolbox is a library and CLI application
that can convert SDO files into PDF files. To make this possible, the
toolbox contains a custom PDF writing library that is in some ways the
opposite of, but also prototype for an extension to [`pdf-rs`].

This library implements some subset of the PDF standard that allows
us to create valid PDF documents with embedded Type3 bitmap fonts
(converted from the Signum! printer fonts) and fine grained control
over the position of each character on the page.

This also allows us to delay the conversion to Unicode codepoints
to the PDF viewer and to keep the encoding of the Signum! fonts.

[`pdf-rs`]: https://crates.io/crates/pdf

## CCITT-T.4-T.6

Part of the SDO-Toolbox is an encoder (and decoder) for the CCITT Group 4
monochrome bitmap coding scheme, that was created for fax machines and
is used in PDFs to store small monochrome bitmap images. SDO-Toolbox
uses that to transform an Signum! Printer font file into an Adobe Type3
font to be used in generated PDF files.

See also: <https://github.com/Xiphoseer/sdo-tool/tree/main/crates/ccitt>  
T.4: <https://www.itu.int/rec/T-REC-T.4>  
T.6: <https://www.itu.int/rec/T-REC-T.6>