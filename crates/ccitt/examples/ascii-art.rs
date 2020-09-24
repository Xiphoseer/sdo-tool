use std::path::PathBuf;

use structopt::StructOpt;
use ccitt_t4_t6::{g42d::decode::Decoder, bit_iter::BitWriter, bit_iter::BitIter};
use color_eyre::eyre;

#[derive(StructOpt)]
struct Options {
    file: PathBuf,
    #[structopt(long)]
    width: usize,
    #[structopt(long, short)]
    invert: bool,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let opt = Options::from_args();
    let file = std::fs::read(&opt.file)?;
    
    let mut decoder = Decoder::<BitWriter>::new(opt.width);
    decoder.decode(&file)?;
    let store = decoder.into_store();

    let bitmap = store.done();
    let iter = BitIter::new(&bitmap);

    print!("+");
    for _ in 0..opt.width {
        print!("-");
    }
    println!("+");
    for (i, bit) in iter.enumerate() {
        let mod_width = i % opt.width;
        if mod_width == 0 {
            print!("|");
        }
        if bit ^ opt.invert {
            print!("#");
        } else {
            print!(" ");
        }
        if mod_width == opt.width - 1 {
            println!("|");
        }
    }
    print!("+");
    for _ in 0..opt.width {
        print!("-");
    }
    println!("+");

    Ok(())
}