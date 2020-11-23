# Changelog

## Unreleased (main)

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