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

### Links

Source: [Xiphoseer/sdo-tool:crates/pdf](https://github.com/Xiphoseer/sdo-tool/tree/main/crates/pdf)  
Crate: <https://crates.io/crates/pdf-create>  
Docs: <https://docs.rs/pdf-create>

### References

Spec: [PDF 32000-1:2008 (v1.7)](https://www.adobe.com/content/dam/acom/en/devnet/pdf/PDF32000_2008.pdf)

[`pdf-rs`]: https://crates.io/crates/pdf

## Signum

The code that is used to load the signum files into memory does not depend
on the rest of the system. This crate contains the datastructures and parser
code for Signum!-related file formats.

There is no definitive spec for that file format and the implementation is
*best-effort* and may not always work correctly. It also requires some
driver like sdo-tool to be really useful as it does not attempt to combine
the formats. For example, it does not attempt to load charset files to get
a glyph for some letter in the text.

### Links

Source: [Xiphoseer/sdo-tool:crates/signum](https://github.com/Xiphoseer/sdo-tool/tree/main/crates/signum)  
Crate: <https://crates.io/crates/signum>  
Docs: <https://docs.rs/signum>

## CCITT-T.4-T.6

Part of the SDO-Toolbox is an encoder (and decoder) for the CCITT Group 4
monochrome bitmap coding scheme, that was created for fax machines and
is used in PDFs to store small monochrome bitmap images. SDO-Toolbox
uses that to transform an Signum! Printer font file into an Adobe Type3
font to be used in generated PDF files.

### Links

Source: [Xiphoseer/sdo-tool:crates/ccitt](https://github.com/Xiphoseer/sdo-tool/tree/main/crates/ccitt)  
Crate: <https://crates.io/crates/ccitt-t4-t6>  
Docs: <https://docs.rs/ccitt-t4-t6>

### References

T.4: <https://www.itu.int/rec/T-REC-T.4>  
T.6: <https://www.itu.int/rec/T-REC-T.6>

## ESC/P

The Signum! [printer drivers](/signum/printer-drivers) themselves print by generating
ESC/P printer commands to print bit images. This crate implements a small subset
of ESC/P parsing, sufficient to reconstruct a full signum page.

### Links

Source: [Xiphoseer/sdo-tool:crates/esc-p](https://github.com/Xiphoseer/sdo-tool/tree/main/crates/esc-p)  
Crate: <https://crates.io/crates/esc-p>  
Docs: <https://docs.rs/esc-p>

### References

Spec: [EPSON ESC/P Reference Manual](https://files.support.epson.com/pdf/general/escp2ref.pdf)
