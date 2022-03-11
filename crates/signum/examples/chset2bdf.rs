use std::{fmt, io, path::PathBuf};

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

    let pset_bytes = std::fs::read(&args.input)?;
    let name = args.input.file_stem().unwrap().to_string_lossy();
    let (_, pset) = chsets::printer::parse_ps24::<nom::error::Error<_>>(&pset_bytes).unwrap();

    let eset_path = args.input.with_extension("E24");
    let eset_bytes = std::fs::read(&eset_path)?;
    let (_, eset) = chsets::editor::parse_eset(&eset_bytes).unwrap();

    chsets::output::bdf::pset_to_bdf(&mut StdoutWriter, &pset, &eset, &name).unwrap();
    Ok(())
}
