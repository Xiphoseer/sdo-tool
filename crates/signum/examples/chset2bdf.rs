use std::{
    fmt,
    io::{self, Read},
    os::unix::prelude::OsStrExt,
    path::PathBuf,
};

use signum::chsets;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    input: PathBuf,
}

struct StdoutWriter;

impl fmt::Write for StdoutWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        print!("{}", s);
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let args = Args::from_args();
    let mut bytes = Vec::new();
    if args.input.as_os_str().as_bytes() == b"-" {
        std::io::stdin().read_to_end(&mut bytes)?;
    } else {
        bytes = std::fs::read(&args.input)?;
    };

    let (_rest, pset) = chsets::printer::parse_ps24(&bytes).unwrap();
    chsets::output::bdf::pset_to_bdf(&mut StdoutWriter, &pset).unwrap();
    Ok(())
}
