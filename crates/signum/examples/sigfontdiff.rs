use std::{convert::TryFrom, path::PathBuf};

use clap::Parser;
use color_eyre::eyre::{eyre, Context};
use nom::Finish;
use signum::{
    chsets::{editor::parse_eset, printer::parse_font, FontKind},
    docs::four_cc,
};

#[derive(Debug, Parser)]
/// Converts a file from Signum! IMC to a Portable Bit-Map
struct Opts {
    /// First file to compare
    left: PathBuf,

    /// Second file to compare
    right: PathBuf,
}

type E<'a> = nom::error::Error<&'a [u8]>;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let opts: Opts = Opts::parse();

    let left = std::fs::read(&opts.left)
        .wrap_err_with(|| format!("Failed to open {}", opts.left.display()))?;
    let right = std::fs::read(&opts.right)
        .wrap_err_with(|| format!("Failed to open {}", opts.right.display()))?;

    let (_, left_four_cc) = four_cc::<E>(&left)
        .finish()
        .map_err(|_| eyre!("File too small to be a font: {}", opts.left.display()))?;

    let font_kind = FontKind::try_from(left_four_cc)?;
    match font_kind {
        FontKind::Editor => {
            let (_, left) = parse_eset(&left)
                .finish()
                .map_err(|_| eyre!("Failed to parse {} as editor font", opts.left.display()))?;
            let (_, right) = parse_eset(&right)
                .finish()
                .map_err(|_| eyre!("Failed to parse {} as editor font", opts.right.display()))?;
            let mut count = 0;
            for (index, (l, r)) in left.chars.iter().zip(right.chars.iter()).enumerate() {
                if l != r {
                    println!("Char {index} differs:");
                    println!("left: {:?}", l);
                    println!("right: {:?}", r);
                    count += 1;
                }
            }
            println!("Total differences: {}", count);
        }
        FontKind::Printer(pk) => {
            let (_, left) = parse_font::<E>(&left, pk)
                .finish()
                .map_err(|_| eyre!("Failed to parse {} as printer font", opts.left.display()))?;
            let (_, right) = parse_font::<E>(&right, pk)
                .finish()
                .map_err(|_| eyre!("Failed to parse {} as printer font", opts.right.display()))?;
            let mut count = 0;
            for (index, (l, r)) in left.chars.iter().zip(right.chars.iter()).enumerate() {
                if l != r {
                    println!("Char {index} differs:");
                    println!("left: {:?}", l);
                    println!("right: {:?}", r);
                    count += 1;
                }
            }
            println!("Total differences: {}", count);
        }
    }

    Ok(())
}
