use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;
use color_eyre::eyre::{self, WrapErr};
use log::{info, LevelFilter};
use pdf_create::{
    common::{PageLabel, PdfString},
    high::{self, Handle},
};
use sdo_pdf::{
    font::Fonts, prepare_info, prepare_pdfa_output_intent, sdoc::generate_pdf_pages, Pdf,
};
use signum::{
    chsets::{cache::ChsetCache, printer::PrinterKind, UseMatrix, UseTableVec},
    util::LocalFS,
};

use sdo_tool::cli::{
    opt::{DocScript, OutlineItem},
    sdoc::{
        pdf::{handle_out, GenCtx},
        Document,
    },
};

#[derive(clap::Parser, Debug)]
/// Run a document script
pub struct RunOpts {
    /// A document script
    file: PathBuf,
    /// The output folder
    #[clap(default_value = ".")]
    out: PathBuf,
}

fn map_outline_items(items: &[OutlineItem]) -> eyre::Result<Vec<high::OutlineItem>> {
    let mut result = Vec::with_capacity(items.len());
    for item in items {
        let title = PdfString::from_str(&item.title)?;
        result.push(high::OutlineItem {
            title,
            dest: item.dest.into(),
            children: map_outline_items(&item.children)?,
        });
    }
    Ok(result)
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

    // Prepare output folder
    if opt.out != Path::new("-") {
        std::fs::create_dir_all(&opt.out)?;
    }

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

    // Preprare output
    let mut hnd = Handle::new();

    prepare_info(&mut hnd.meta, &script.meta.to_pdf_meta())?;
    prepare_pdfa_output_intent(&mut hnd)?;

    let mut use_table_vec = UseTableVec::new();
    let mut use_table_vec_bold = UseTableVec::new();
    for (doc, di) in &documents {
        let pages = doc.text_pages();
        let use_matrix = UseMatrix::of_matching(pages, |k| !k.style.is_bold());
        let use_matrix_bold = UseMatrix::of_matching(pages, |k| k.style.is_bold());
        use_table_vec.append(&di.fonts, use_matrix);
        use_table_vec_bold.append(&di.fonts, use_matrix_bold);
    }

    // FIXME: Auto-Detect from font cache
    let pk = PrinterKind::Laser30;

    let fonts_capacity = fc.chsets().len();
    let mut font_info = Fonts::new(fonts_capacity);

    font_info.make_fonts(&fc, &mut hnd.res, use_table_vec, use_table_vec_bold, pk);

    let overrides = script.meta.to_overrides();
    for (doc, di) in &documents {
        let gc = GenCtx::new(doc, di);
        generate_pdf_pages(&gc, &mut hnd, &overrides, &font_info)?;
    }

    for (key, value) in &script.page_labels {
        let prefix = PdfString::from_str(&value.prefix)?;
        hnd.page_labels.insert(
            *key,
            PageLabel {
                prefix,
                kind: value.kind.into(),
                start: value.start,
            },
        );
    }

    hnd.outline.children = map_outline_items(&script.outline)?;

    handle_out(Some(&opt.out), &opt.file, Pdf::from_raw(hnd))?;
    Ok(())
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(LevelFilter::Info)
        .init();
    let opt: RunOpts = RunOpts::parse();

    let file_res = File::open(&opt.file);
    let file = WrapErr::wrap_err_with(file_res, || {
        format!("Failed to open file: `{}`", opt.file.display())
    })?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    run(&buffer, opt)
}
