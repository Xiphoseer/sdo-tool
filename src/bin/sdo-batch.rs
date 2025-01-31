use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use clap::Parser;
use color_eyre::eyre::{self, WrapErr};
use log::{info, LevelFilter};
use pdf_create::{
    common::{PageLabel, PdfString},
    encoding::{pdf_doc_encode, pdf_doc_encode_lossy},
    high::{self, Handle},
};
use sdo_pdf::font::Fonts;
use signum::chsets::{cache::ChsetCache, UseTableVec};

use sdo_tool::cli::{
    opt::{DocScript, OutlineItem},
    sdoc::{
        pdf::{self, AutoOutline},
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

fn map_outline_items(items: &[OutlineItem]) -> Vec<high::OutlineItem> {
    let mut result = Vec::with_capacity(items.len());
    for item in items {
        let title = PdfString::new(pdf_doc_encode_lossy(&item.title));
        result.push(high::OutlineItem {
            title,
            dest: item.dest.into(),
            children: map_outline_items(&item.children),
        });
    }
    result
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
    let mut fc = ChsetCache::new(chsets_folder);

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
        document.process_sdoc(input, &mut fc)?;
        documents.push(document);
    }

    // Preprare output
    let mut hnd = Handle::new();

    pdf::prepare_meta(&mut hnd, &script.meta)?;

    let mut use_table_vec = UseTableVec::new();
    for doc in &documents {
        let use_matrix = doc.use_matrix();
        use_table_vec.append(&doc.chsets, use_matrix);
    }

    let pk = fc
        .print_driver(None)?
        .printer()
        .ok_or_else(|| eyre::eyre!("Printing with editor font not supported in PDF"))?;

    // Prepare the fonts
    let fonts_capacity = fc.chsets().len();
    let mut font_info = Fonts::new(fonts_capacity, hnd.res.fonts.len());

    let fonts = font_info.make_fonts(&fc, use_table_vec, pk);
    pdf::push_fonts(&mut hnd, fonts);

    // Prepare the auto outline
    let mut auto_outline = AutoOutline::new(
        &script.auto_outline.levels,
        script.auto_outline.min_line_index,
    )?;
    auto_outline.req_same_style(script.auto_outline.req_same_style);
    log::info!("{:?}", script.auto_outline.toc);
    if let Some(tt) = &script.auto_outline.toc {
        let page_range = tt.page_range.0..(tt.page_range.1 + 1);
        auto_outline.set_auto_toc(&tt.title, page_range);
    }

    // Loop over all documents / pages
    let mut page_count = 0;
    for doc in &documents {
        pdf::prepare_document(
            &mut hnd,
            doc,
            page_count,
            &script.meta,
            &font_info,
            &mut auto_outline,
        )?;
        page_count += doc.page_count();
    }

    for (key, value) in &script.page_labels {
        let prefix = PdfString::new(pdf_doc_encode(&value.prefix)?);
        hnd.page_labels.insert(
            *key,
            PageLabel {
                prefix,
                kind: value.kind.into(),
                start: value.start,
            },
        );
    }

    // Generate Outline (either automatic or user-defined)
    let outline = if script.outline.is_empty() {
        auto_outline.get_items()
    } else {
        &script.outline
    };
    hnd.outline.children = map_outline_items(outline);

    // Write the PDF file
    pdf::handle_out(Some(&opt.out), &opt.file, hnd)?;
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
