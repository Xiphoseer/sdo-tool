use std::path::PathBuf;

use prettytable::{cell, format, row, Cell, Row, Table};
use signum::chsets::encoding::p_mapping_file;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opts {
    file: PathBuf,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let opts = Opts::from_args();
    let input = std::fs::read_to_string(opts.file)?;
    let mapping = p_mapping_file(&input)?;

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    // Add a row per time
    table.set_titles(row![
        "", "_0", "_1", "_2", "_3", "_4", "_5", "_6", "_7", "_8", "_9", "_a", "_b", "_c", "_d",
        "_e", "_f"
    ]);

    for (index, chars) in mapping.chars.chunks(16).enumerate() {
        let mut cells = Vec::with_capacity(16);
        cells.push(Cell::new(&format!("{:x}_", index)));
        for chr in chars {
            if *chr == '\0' {
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
