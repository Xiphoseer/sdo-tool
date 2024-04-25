use nom::{Finish, Parser};
use nom_supreme::{
    error::ErrorTree,
    final_parser::{ByteOffset, ExtractContext},
};
use std::path::PathBuf;
use structopt::StructOpt;
use texfonts::pk::{Decoder, Event};

#[derive(Debug, StructOpt)]
/// Prints information about a X11 PCF file
struct Opts {
    file: PathBuf,
}

fn to_err_tree<'a>(
    original_input: &'a [u8],
) -> impl FnOnce(ErrorTree<&'a [u8]>) -> ErrorTree<usize> {
    move |t| {
        let t2: ErrorTree<ByteOffset> = t.extract_context(original_input);
        let t3: ErrorTree<usize> = t2.map_locations(|o| o.0);
        t3
    }
}

fn load<'a, F, T>(mut fun: F, input: &'a [u8]) -> Result<(&'a [u8], T), ErrorTree<usize>>
where
    F: Parser<&'a [u8], T, ErrorTree<&'a [u8]>>,
{
    fun.parse(input).finish().map_err(to_err_tree(input))
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let opts: Opts = Opts::from_args();
    let buffer = std::fs::read(opts.file)?;

    let decoder: Decoder<'_, ErrorTree<&[u8]>> = Decoder::new(&buffer);
    for event in decoder {
        match event {
            Ok(Event::Command(cmd)) => println!("{:?}", cmd),
            Ok(Event::Character(chr)) => {
                println!(
                    "Character {{ dyn_f: {}, frb: {:?}, cc: {}, fl: {:?} }}",
                    chr.dyn_f, chr.first_run_black, chr.cc, chr.fl
                );
                let (raster, pre) = load(chr.fl, chr.bytes)?;
                println!("{:?}", pre);

                for row in raster.chunks(16) {
                    print!("   ");
                    for byte in row {
                        print!(" {:02x}", byte);
                    }
                    println!();
                }
            }
            Err(e) => {
                let map_err = to_err_tree(&buffer);
                return Err(match e {
                    nom::Err::Incomplete(_) => panic!(),
                    nom::Err::Error(e) | nom::Err::Failure(e) => map_err(e).into(),
                });
            }
        }
    }

    Ok(())
}
