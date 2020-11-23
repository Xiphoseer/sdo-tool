# Implementation

The SDO-Toolbox is a CLI Application written in the
[Rust Programming Language](https://rust-lang.org).

## CCITT T.4 T.6

Part of the SDO-Toolbox is an encoder (and decoder) for the CCITT Group 4
monochrome bitmap coding scheme, that was created for fax machines and
is used in PDFs to store small monochrome bitmap images. SDO-Toolbox
uses that to transform an Signum! Printer font file into an Adobe Type3
font to be used in generated PDF files.

See also: <https://github.com/Xiphoseer/sdo-tool/tree/main/crates/ccitt>