use std::path::PathBuf;

use ccitt_t4_t6::{bit_iter::BitIter, bit_iter::BitWriter, g42d::decode::Decoder};
use color_eyre::eyre;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    file: PathBuf,
    #[structopt(long)]
    width: usize,
    #[structopt(long, short)]
    invert: bool,
    #[structopt(long)]
    debug: bool,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let opt = Options::from_args();
    let file = std::fs::read(&opt.file)?;

    let mut decoder = Decoder::<BitWriter>::new(opt.width);
    decoder.debug = opt.debug;
    decoder.decode(&file)?;
    let store = decoder.into_store();

    let bitmap = store.done();
    let mut iter = BitIter::new(&bitmap);

    let width = opt.width;
    let height = bitmap.len() * 8 / width;

    print!("+");
    for _ in 0..opt.width {
        print!("-");
    }
    println!("+");
    for _ in 0..height {
        print!("|");
        for _ in 0..width {
            let bit = iter.next().unwrap();
            if bit ^ opt.invert {
                print!("#");
            } else {
                print!(" ");
            }
        }
        println!("|");
    }
    print!("+");
    for _ in 0..opt.width {
        print!("-");
    }
    println!("+");

    Ok(())
}
