# Changelog

## Unreleased (`main` branch)

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