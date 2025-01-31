use std::{fs::File, io::BufWriter, io::Write, path::PathBuf};

use clap::Parser;
use color_eyre::eyre::{self, eyre};

#[derive(Parser)]
struct Opt {
    from: PathBuf,
    to: PathBuf,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let opt = Opt::parse();
    let buffer = std::fs::read(&opt.from)?;
    if buffer.starts_with(b"crypted2") {
        let buffer = &buffer[8..];
        let (magic, b) = buffer.split_at(8);
        let (count, d) = b.split_at(2);
        let (base, dat) = d.split_at(128 + 6);

        let f = File::create(&opt.to)?;
        let mut w = BufWriter::new(f);
        w.write_all(magic)?;
        w.write_all(count)?;
        w.write_all(base)?;
        for byte in dat {
            // FIXME: This was a wild guess and is not actually the algorithm that was used
            let low_p = *byte & 0xf;
            let high_p = *byte >> 4;

            let low_m = low_p + 15;
            let high_m = high_p + 15;

            let low = low_m & 0xf;
            let high = high_m & 0xf;
            println!(
                "{:04b} {:04b} -> {:04b} {:04b} -> {:04b} {:04b}",
                high_p, low_p, high_m, low_m, high, low
            );

            let dec = low | (high << 4);
            w.write_all(&[dec])?;
        }
        w.flush()?;
        Ok(())
    } else {
        Err(eyre!("Not a `crypted2` file"))
    }
}
