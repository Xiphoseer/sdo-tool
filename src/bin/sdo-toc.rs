use std::{fs, path::PathBuf};

use clap::Parser;
use color_eyre::eyre::{self, WrapErr};
use log::info;
use signum::{chsets::cache::ChsetCache, util::LocalFS};

use sdo_tool::cli::{self, opt::DocScript, sdoc::Document};

#[derive(Parser, Debug)]
/// Run a document script
pub struct RunOpts {
    /// A document script
    file: PathBuf,
}

pub fn run(buffer: &[u8], opt: RunOpts) -> eyre::Result<()> {
    let script_str_res = std::str::from_utf8(buffer);
    let script_str = WrapErr::wrap_err(script_str_res, "Failed to parse as string")?;
    let script_res = ron::from_str(script_str);
    let script: DocScript = WrapErr::wrap_err(script_res, "Failed to parse DocScript")?;

    // Set-Up font cache
    let folder = opt.file.parent().unwrap();
    let chsets_folder = folder.join(&script.chsets);
    let chsets_folder: PathBuf = chsets_folder.canonicalize().wrap_err_with(|| {
        format!(
            "Failed to canonicalize CHSETS folder path `{}`",
            chsets_folder.display()
        )
    })?;
    let fs = LocalFS::new(chsets_folder);
    let mut fc = ChsetCache::new();

    let capacity = script.files.len();

    // Load documents
    let mut doc_files = Vec::with_capacity(capacity);
    for doc_path in &script.files {
        let doc_file = folder.join(doc_path);
        let doc_file = doc_file.canonicalize().wrap_err_with(|| {
            format!(
                "Failed to canonicalize document file path `{}`",
                doc_file.display()
            )
        })?;
        let input = std::fs::read(&doc_file)?;
        doc_files.push((doc_file, input));
    }

    let mut documents = Vec::with_capacity(capacity);
    for (doc_file, input) in &doc_files {
        let mut document = Document::new();

        info!("Loading document file '{}'", doc_file.display());
        let di = document.process_sdoc(input, &fs, &mut fc)?;
        documents.push((document, di));
    }

    Ok(())
}

fn main() -> color_eyre::Result<()> {
    let opt: RunOpts = cli::init()?;

    let buffer = fs::read(&opt.file)
        .wrap_err_with(|| format!("Failed to open file: `{}`", opt.file.display()))?;

    run(&buffer, opt)
}
