# Signum! Document Toolbox

The *Signum!* program was a popular word-processor available for the ATARI AT line of
home computers and distributed by [Application Systems Heidelberg (ASH)][ASH]. While it
could be used via the *MagicCMac* or *MagicCPC* emulators, it was never ported to newer
systems. The file format (*.SDO for Signum! and Signum!2) was proprietary or at least
unspecified and with the exception of *Papyrus* for the ATARI ST and a small amount of
other for-profit tools, no other software could read those files. When I started this
project, I could not find any software that runs on modern systems and can read those
files.

This is especially unfortunate for people (mostly german-speaking) that wrote their
thesis in Signum! in the 80s and still have the floppy disks with those files but no
ATARI and/or no application anymore. Yes, you can set-up an emulator, get the installation
files from ASH, use or [buy a product key][Signum!], create/insert virtual floppy disks,
install Signum!, open your files, export the text to ASCII, export the images, use
vision/zView for ATARI to covert the images, fix up umlauts in the ATARI-ASCII, and be
done with one document, but that is a lot to ask for and I haven't seen a single post
mention all of this at once. And it's really not worth the effort for some of the less
important stuff I've found in the stash of floppies I had access to.

Or you can use this (work-in-progress) tool, which can read Signum!2 documents and
make some of these steps much easier. It's written in Rust, so it should work across
all major platforms (Windows, Linux and OSX).

## Features

- Load Signum! 1/2 documents (*.SDO, `sdoc0001`)
    - Print all kinds of data from the files to a console, but most importantly
    - Print a list of charset names
    - Print some global formatting options
    - Print a list of pages with additional formatting information
    - Print text to the console, including page breaks, and some HTML-like format annotations
        - Fonts that match the default ATARI keys are translated to the appropriate unicode characters
        - Documents that use only the ANTIKRO font have working space detection. This is a WIP
    - Print a list of image names
- Load Signum! editor charsets (*.E24, `eset0001`)
    - Print height and width for each character
    - Print ASCII art for each character bitmap
- Load Signum! images (*.IMC, `bimc0002`)
    - Produce a PNG for monochrome images exported from a document
    - NOTE: It's a tiny step from there to exporting images from a document on the fly.

## Usage

1. Install [Rust][Rust]
2. Download/Clone this repository
3. In the folder that contains `Cargo.toml` run `cargo build --release` / `cargo.exe build --release`
4. Use/Copy `target/release/sdo-tool` (Linux/OSX) or `target/release/sdo-tool.exe` (Windows)
5. Run `sdo-tool SOMEFILE.SDO`, `sdo-tool SOMEFILE.E24`, `sdo-tool SOMEFILE.IMC`

## Examples

See [this document](https://xiphoseer.github.io/sdo-tool/examples.html)

## Format Documentation

- [Signum! Document (`sdoc0001`)](https://xiphoseer.github.io/sdo-tool/format-sdoc.html)
- [Editor Charset (`eset`)](https://xiphoseer.github.io/sdo-tool/format-eset.html)

## License

You are free to download, and run this software for any personal and non-commercial reason,
at your own risk and without claiming fitness for any particular purpose. If you want to
contribute to this project or use it for any kind of distribution or commercial purpose,
please file a github issue or send me an email at `ng5on06lc@relay.firefox.com`.

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
[Signum!]: https://www.ashshop.biz/diverses/atari/textverarbeitung/874/signum-2-download
[Rust]: https://www.rust-lang.org/learn/get-started
[image-rs]: https://crates.io/crates/image
[MM2]: http://www.tempel.org/files-d.html#MM2
[ZERO-WHITE]: https://en.wikipedia.org/wiki/List_of_monochrome_and_RGB_color_formats#Monochrome_(1-bit)
[1stWord]: http://stcarchiv.de/stc1989/02/von-1stword-zu-signum2