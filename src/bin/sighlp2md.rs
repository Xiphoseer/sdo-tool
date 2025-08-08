use clap::Parser;
use signum::help;
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

#[derive(Parser, Debug)]
#[clap(
    name = "sighlp2md",
    version = "0.1.0",
    about = "Convert Signum 3/4 .HLP files to Markdown"
)]
/// Convert Signum 3/4 .HLP files to Markdown
struct Options {
    /// Input .HLP file
    input: PathBuf,
    /// Output directory for Markdown files
    output: PathBuf,
    /// Only print what would be written, do not modify the filesystem
    #[clap(long)]
    dry_run: bool,
}

fn main() -> io::Result<()> {
    let Options {
        input,
        output,
        dry_run,
    } = Options::parse();

    let help_file = help::HelpFile::read_help_file(&input)?;
    write_help_file(&help_file, &output, dry_run)?;

    Ok(())
}

fn write_help_file(help_file: &help::HelpFile, output: &Path, dry_run: bool) -> io::Result<()> {
    for (term, tdata) in &help_file.terms {
        // Main term file: output/term.md
        let term_path = output.join(term).with_extension("md");
        write_content_file(
            &term_path,
            tdata.title.as_deref().unwrap_or(term),
            &tdata.content,
            dry_run,
        )?;

        // Subterms
        let subterm_dir = output.join(term);
        for (sub, sdata) in &tdata.subterms {
            // Subterm file: output/term/sub.md
            let sub_path = subterm_dir.join(sub).with_extension("md");
            write_content_file(
                &sub_path,
                sdata.title.as_deref().unwrap_or(sub),
                &sdata.content,
                dry_run,
            )?;
            // Subterm aliases: output/term/alias.md
            for alias in &sdata.aliases {
                let alias_path = subterm_dir.join(alias).with_extension("md");
                let redirect_to = format!("/{}/{}", term, sub);
                write_alias_file(alias_path, &redirect_to, dry_run)?;
            }
        }
        // Term aliases: output/alias.md
        for alias in &tdata.aliases {
            let alias_path = Path::new(output).join(alias).with_extension("md");
            let redirect_to = format!("/{}", term);
            write_alias_file(alias_path, &redirect_to, dry_run)?;
        }
    }

    Ok(())
}

fn write_content_file<P: AsRef<std::path::Path>>(
    path: P,
    title: &str,
    content: &[String],
    dry_run: bool,
) -> io::Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        if !dry_run {
            fs::create_dir_all(parent)?;
        } else {
            eprintln!("Would create directory: {}", parent.display());
        }
    }
    if !dry_run {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        write!(file, "---\ntitle: \"{}\"\n---\n\n", title)?;
        for line in content {
            writeln!(file, "{}", line)?;
        }
    } else {
        eprintln!("Would write content file: {}", path.as_ref().display());
    }
    Ok(())
}

fn write_alias_file<P: AsRef<std::path::Path>>(
    path: P,
    redirect_to: &str,
    dry_run: bool,
) -> io::Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        if !dry_run {
            fs::create_dir_all(parent)?;
        } else {
            eprintln!("Would create directory: {}", parent.display());
        }
    }
    if !dry_run {
        let mut alias_md = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        write!(alias_md, "---\nredirect_to: {}\n---\n", redirect_to)?;
    } else {
        eprintln!("Would write alias file: {}", path.as_ref().display());
    }
    Ok(())
}
