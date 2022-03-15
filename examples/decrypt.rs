use std::{fs::File, io::BufWriter, io::Write, path::PathBuf};

use clap::Parser;
use color_eyre::eyre::{self, eyre};
use signum::chsets::editor::parse_echar;

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
        let (count, d) = b.split_at(4);
        let (base, e) = d.split_at(128);
        let (num, dat) = e.split_at(4);
        println!("{count:?}, {base:?}, {num:?}");

        let f = File::create(&opt.to)?;
        let mut w = BufWriter::new(f);
        w.write_all(magic)?;
        w.write_all(count)?;
        w.write_all(base)?;
        w.write_all(num)?;

        /*
        let num = u32::from_be_bytes([num[0], num[1], num[2], num[3]]) as usize;
        let (_ofs, _rest) = dat.split_at(num);
        for chnk in _ofs.chunks(4).take(128) {
            let v = [chnk[0], chnk[1], chnk[2], chnk[3]];
            let v = (u32::from_be_bytes(v) ^ 0x11111111) as usize;
            eprintln!("{chnk:?} => {v:x}");
        }
        */

        let (_, rest) = dat.split_at(128 * 4);
        let mut buf = Vec::with_capacity(rest.len());
        for b in rest {
            buf.push(b ^ 0x11);
        }
        let mut rest = &buf[..];
        for i in 1..127 {
            let (r, e) = parse_echar(rest).unwrap();
            eprintln!("{i}: {e:?}");
            e.print();
            rest = r;
        }

        for byte in dat {
            w.write_all(&[byte ^ 0x11])?;
            /*
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
            w.write_all(&[dec])?; */
        }
        w.flush()?;
        Ok(())
    } else {
        Err(eyre!("Not a `crypted2` file"))
    }
}
