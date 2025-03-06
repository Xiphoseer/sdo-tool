use std::path::PathBuf;

use clap::Parser;
use prettytable::{format, row, Cell, Row, Table};
use signum::chsets::encoding::{p_mapping_file, Mapping};

#[derive(Debug, Parser)]
pub struct Opts {
    file: PathBuf,
    #[clap(short, long)]
    code: bool,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let opts = Opts::parse();
    let input = std::fs::read_to_string(opts.file)?;
    let mapping = p_mapping_file(&input)?;

    if opts.code {
        print_code(&mapping)?;
    } else {
        print_table(&mapping)?;
    }
    Ok(())
}

fn print_code(mapping: &Mapping) -> color_eyre::Result<()> {
    let mut string = String::new();
    signum::chsets::code::write_map(mapping, &mut string, "MAP")?;
    print!("{string}");
    Ok(())
}

fn print_table(mapping: &Mapping) -> color_eyre::Result<()> {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    // Add a row per time
    table.set_titles(row![
        "", "_0", "_1", "_2", "_3", "_4", "_5", "_6", "_7", "_8", "_9", "_a", "_b", "_c", "_d",
        "_e", "_f"
    ]);

    let mut iter = mapping.chars();
    for index in 0..8usize {
        let mut cells = Vec::with_capacity(16);
        let chars = (&mut iter).take(16);
        cells.push(Cell::new(&format!("{:x}_", index)));
        for chr in chars {
            if *chr == ['\0'] || *chr == [char::REPLACEMENT_CHARACTER] {
                cells.push(Cell::new(""));
            } else {
                let cdisp = format!("{:?}", chr);
                cells.push(Cell::new(&cdisp));
            }
        }
        table.add_row(Row::new(cells));
    }

    // Print the table to stdout
    table.printstd();

    Ok(())
}
