# Signum! Document Toolbox

The *Signum!* program was a popular word-processor available for the ATARI ST line of
home computers and distributed by [Application Systems Heidelberg (ASH)][ASH]. While it
could be used via the *MagicCMac* or *MagicCPC* emulators, it was never ported to newer
systems. The file format (*.SDO for Signum! and Signum!2) was proprietary or at least
unspecified and with the exception of *Papyrus* for the ATARI ST and a small amount of
other for-profit tools, no other software could read those files. When I started this
project, I could not find any software that runs on modern systems and can read those
files.

This is especially unfortunate for people (mostly german-speaking) that wrote their
thesis in Signum! in the 80s and still have the floppy disks with those files but no
ATARI and/or no application anymore. The current solutions are really not worth the
effort for some of the less important stuff I've found in the stash of floppies I had access to.

Instead, you can use this (work-in-progress) tool, which can read Signum!2 documents and
make some of these steps much easier. It's written in Rust, so it should work across
all major platforms (Windows, Linux and OSX).

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [License](#license)
- [Contact](#contact)
- [Acknowledgements](#acknowledgements)

## Features

- Load Signum! 1/2 documents (`*.SDO`)
    - Print all kinds of data from the files to a console, but most importantly
    - Print a list of charset names
    - Print some global formatting options
    - Print a list of pages with additional formatting information
    - Export the document as PDF or HTML or one PNG image per page
        - See [examples][examples]
        - This requires the E24 font files to be available
        - Font modes (e.g. bold and italic) are not yet supported in all formats
    - Print a list of image names
- Load Signum! editor charsets (`*.E24`)
    - Print height and width for each character
    - Print ASCII art for each character bitmap
    - Render a string with the font (use `--input "..."`)
        - currently always "Abcdefghijkl"
    - Generate a visual map of the characters on an ATARI keyboard
        - Examples of those are linked on the [chsets][chsets] page
- Load Signum! printer charsets (`*.P24`, `*.P09`, `*.L30`)
    - Print height and width for each character
    - Print ASCII art for each character bitmap
- Load Signum! images (`*.IMC`, `*.I__`)
    - Produce a PNG for monochrome images exported from a document

## Installation

This project is implemented in the [Rust][Rust] programming language. You can either
download the compiled executables (recommended) or build the program from source using
`cargo`.

### Download (recommended)

1. Go to <https://github.com/Xiphoseer/sdo-tool/releases>
2. Download "sdo-toolbox-XXX.zip" where XXX is your operating system
3. Unzip the archive to `sdo-toolbox-XXX`
4. Use/Copy `sdo-toolbox/sdo-tool` (Linux/OSX) or `sdo-toolbox/sdo-tool.exe` (Windows)

### From Source

1. Install [Rust][Rust]
2. Clone this repository (`git clone git@github.com:Xiphoseer/sdo-tool.git`)
3. OR download it from <https://github.com/Xiphoseer/sdo-tool/archive/main.zip>
4. In the folder that contains `Cargo.toml` run `cargo build --release` / `cargo.exe build --release`
5. Use/Copy `target/release/sdo-tool` (Linux/OSX) or `target/release/sdo-tool.exe` (Windows)

### Adding to the path

On Linux and MacOS, you can copy `sdo-tool` to `/usr/local/bin` to have it available everywhere.
On all operating systems, you can add the `sdo-toolbox` or `release` folder to the `PATH` environment variable (See [this guide](https://www3.ntu.edu.sg/home/ehchua/programming/howto/Environment_Variables.html)) for the same result.

### From crates.io

At the moment, this program is not released on <https://crates.io>, so it cannot be installed using `cargo install` yet.

## Usage

1. `sdo-tool --format pdf SOMEFILE.SDO` to get a PDF file
2. `sdo-tool --format png SOMEFILE.SDO` to get a sequence of PNG files
3. `sdo-tool SOMEFILE.SDO` to print some text to the console
4. `sdo-tool SOMEFILE.E24` to print all characters in the font to the console
5. `sdo-tool --format png SOMEFILE.IMC` to convert an IMC image file to a PNG file
5. `sdo-tool --format pbm SOMEFILE.IMC` to convert an IMC image file to a PBM file

### Example

```sh
$ ls
FILE.SDO
CHSETS
sdo-toolbox
$ cd sdo-toolbox
$ ./sdo-tool --format pdf ../FILE.SDO
...
$ open ../FILE.pdf
```

See [this document][examples] for examples for the output that this tool produces.

## License

You are free to download, and run this software for any personal and non-commercial reason,
at your own risk and without claiming fitness for any particular purpose. If you want to
contribute to this project or use it for any kind of distribution or commercial purpose,
please contact me.

## Contact 

If you had some reason to use this tool, I'd love to hear your story! Please reach out to
me at `xiphoseer@mailbox.org`.

<a href='https://ko-fi.com/Z8Z53PQWH' target='_blank'><img height='36' style='border:0px;height:36px;' src='https://cdn.ko-fi.com/cdn/kofi5.png?v=2' border='0' alt='Buy Me a Coffee at ko-fi.com' /></a>

## Acknowledgements

- [Lonny Pursell](http://atari.gfabasic.net/) for helping me figure out the image reading
  based on his zView-compatible codec
- Oliver Buchmann of [Application Systems Heidelberg](https://ashshop.biz) for helping
  me set up Signum in an emulator.
- [Thomas Tempelmann](http://tempel.org) for the [Megamax-2 Doku][MM2]
  as example files (with images!) and a *SignumRead* (`S/MYUTIL/SIGNUMRE.M`)
  Program from his MM2 dev-environment
- Georg Scheibler for his [article and source code for converting 1stWord to Signum][1stWord]
- The [image-rs] Developers for the PNG generation library
- "The 68000's Instruction Set", which is excellent but sadly doesn't include an author
- Wikipedia editors for confirming that [0 means white][ZERO-WHITE] for an ATARI ST screen dump
- The rust compiler + ecosystem people for creating a useful language with really good tooling
  which made porting 68K assembly to working rust code really smooth

[ASH]: https://application-systems.de

[Rust]: https://www.rust-lang.org/learn/get-started
[image-rs]: https://crates.io/crates/image
[MM2]: http://www.tempel.org/files-d.html#MM2
[ZERO-WHITE]: https://en.wikipedia.org/wiki/List_of_monochrome_and_RGB_color_formats#Monochrome_(1-bit)
[1stWord]: http://stcarchiv.de/stc1989/02/von-1stword-zu-signum2

[examples]: https://xiphoseer.github.io/sdo-tool/examples.html
[chsets]: https://xiphoseer.github.io/sdo-tool/chsets/index.html