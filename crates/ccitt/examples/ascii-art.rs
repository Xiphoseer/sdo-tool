use std::path::PathBuf;

use ccitt_t4_t6::{bits::BitWriter, g42d::Decoder};
use color_eyre::eyre;

#[derive(argh::FromArgs)]
/// load a Group 4 encoded file and write it to console
struct Options {
    #[argh(positional)]
    /// path to input file
    file: PathBuf,
    #[argh(option)]
    /// assume width of the image
    width: usize,
    #[argh(switch)]
    /// invert black and white
    invert: bool,
    #[cfg(feature = "debug")]
    #[argh(switch)]
    /// print debug information
    debug: bool,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let opt: Options = argh::from_env();
    let file = std::fs::read(&opt.file)?;

    let mut decoder = Decoder::<BitWriter>::new(opt.width);
    #[cfg(feature = "debug")]
    {
        decoder.debug = opt.debug;
    }
    decoder.decode(&file)?;
    let store = decoder.into_store();

    let bitmap = store.done();

    let mut string = String::new();
    ccitt_t4_t6::ascii_art(&mut string, &bitmap, opt.width, opt.invert).unwrap();
    print!("{}", string);

    Ok(())
}
