use std::{
    fmt,
    fs::File,
    io::{self, BufWriter, Write},
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

struct FileWriter(BufWriter<File>);

impl fmt::Write for FileWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

fn main() -> io::Result<()> {
    let args = Args::from_args();

    let pset_bytes = std::fs::read(&args.input)?;
    let name = args.input.file_stem().unwrap().to_string_lossy();
    let (_, pset) = chsets::printer::parse_ps24(&pset_bytes).unwrap();

    let eset_path = args.input.with_extension("E24");
    let eset_bytes = std::fs::read(&eset_path)?;
    let (_, eset) = chsets::editor::parse_eset(&eset_bytes).unwrap();

    let bdf_path = args.input.with_extension("bdf");
    let bdf_file = BufWriter::new(File::create(&bdf_path)?);

    chsets::output::bdf::pset_to_bdf(&mut FileWriter(bdf_file), &pset, &eset, &name).unwrap();
    Ok(())
}
