# Changelog

This page lists updates to the SDO-Toolbox.
The name in brackets indicates the relevant crate.

## Version 0.4.x (dev)

### 28.01.2025

- `sdo-web`: Added SDO Studio (https://sdo.dseiler.eu/studio/)
- `sdo-web`: Added full printer font glyph matrix
- `sdo-web`: Added in-browser font collection
- `sdo-pdf`: Moved more of the PDF generation into the crate
- `sdo-pdf`: Support italics (using font matrix)
- `sdo-pdf`: Fixed PDF showing 'A' for some space characters

## Version 0.3.1

### 28.02.2021

- Minor improvements to the CLI experience
- Converting (B)IMC files now requires a `--format` argument

### 26.02.2021

- Initial support for images in PDFs
- Introduce `log` crate for output to the console
- Move chunk tables to delayed console output, requires `--format plain` now

### 23.02.2021

- &#91;sdo-pdf&#93; Added a proper `FontBBox` to resolve issues with Acrobat Reader
- &#91;sdo-pdf&#93; Added Adobe CMap generation
- &#91;mappings&#93; Completed `MATHEM` mapping

### 22.02.2021

- &#91;sdo-tool&#93; Use mapping files in HTML export
- &#91;signum&#93; Added Unicode mapping files (Table A) parsing

### 21.02.2021

- &#91;signum&#93; Added visible page number column to `pbuf` table
- &#91;signum&#93; Initial HTML export (use `--format html`)

### 20.02.2021

- &#91;sdo-pdf&#93; Re-work PDF font generation and positioning

## 19.02.2021

- &#91;texfonts&#93; Add initial PK-Font decoder library

## Version 0.2

### 18.02.2021

- Split out sub-commands into separate executables:
    - `<file> keyboard` &rarr; `chset-kb <file>`
    - `<file> run` &rarr; `sdo-batch <file>`
    - `<file> info` &rarr; `signum-file <file>`
    - `<file> decode` &rarr; `st-decode <file>`
- Improve `--help` descriptions

### 15.09.2020

- The output folder parameter on `dump` was changed from a long argument `--out`
  to a required positional one (`<out>`). If you want to print to the console
  anyways, use the string `-` for the path.
- Removed the `--plain` parameter.
- Added the `--format` parameter. Valid options include `html`, `plain`, `ps`,
  `png` and `pdraw`.
- Added a PostScript output. Currently only working for `L30` fonts, this output
  routine creates a PS file with embedded bitmap fonts (Adobe Type 3). You can
  use this to create a PDF using the `ps2pdf` program from `ghostscript`.

### 12.09.2020 (Preview v0.2)

- Added `--plain` option to the `dump` command to skip printing
  tags that provide debugging information on lines and paragraphs.
- Minor improvements in image positioning and scaling.

### 11.09.2020

- Added recursive font search in `CHSETS` folder. Signum! doesn't do that,
  but it's useful if you have a large collection of fonts and want to find
  out whether you have the needed ones somewhere.

### 10.09.2020

- Added `--page`/`-p` option to `dump`. Optional, can be used multiple times.
  Requires one logical page index as an argument. If used one or
  more times, only those pages are rendered to file.
- Rendering a page to file (`<FILE> dump --out <OUT_DIR>`) now renders
  embedded images as well. This is WIP and positioning might not be perfect
  in all cases yet.