# ccitt-t4-t6

This crate aims to implement the *T.4* and *T.6* standard of the ITU’s
Telecommunication Standardization Sector (ITU-T). These documents
decribe a family of encoding/decoding algorithms for monochrome i.e.
black-and-white bitmaps, which have been created for fax machines.

They are also used in the Adobe PDF file format for the encoding
of monochrome bitmap images as the `/CCITTFaxDecode` stream filter.

## State of implementation

This crate currently implements an Encoder and a Decoder for
the Group 4 2D coding scheme specified in *T.6*. This is sufficient
for *creating* PDF documents that use `CCITTFaxDecode` to draw
Type3 font glyphs. If you want to support non-negative values of
the `K` decode parameter, i.e. Group 3 encoding, feel free to submit
a Pull Request.