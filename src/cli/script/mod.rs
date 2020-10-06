use std::path::{Path, PathBuf};

use color_eyre::eyre::{self, WrapErr};
use pdf::primitive::PdfString;
use pdf_create::{
    common::PageLabel,
    encoding::pdf_doc_encode,
    high::{self, Handle},
};
use sdo::font::{printer::PrinterKind, UseTableVec};
use structopt::StructOpt;

use super::{
    font::cache::FontCache, opt::DocScript, opt::Format, opt::Meta, opt::Options, opt::OutlineItem,
    sdoc::pdf::handle_out, sdoc::pdf::prepare_document, sdoc::pdf::prepare_meta, sdoc::pdf::Fonts,
    sdoc::Document,
};

#[derive(StructOpt, Debug)]
pub struct RunOpts {
    out: PathBuf,
}

fn map_outline_items(items: &[OutlineItem]) -> eyre::Result<Vec<high::OutlineItem>> {
    let mut result = Vec::with_capacity(items.len());
    for item in items {
        let title = PdfString::new(pdf_doc_encode(&item.title)?);
        result.push(high::OutlineItem {
            title,
            dest: item.dest.into(),
            children: map_outline_items(&item.children)?,
        });
    }
    Ok(result)
}

pub fn run(file: PathBuf, buffer: &[u8], opt: RunOpts) -> eyre::Result<()> {
    let script_str_res = std::str::from_utf8(buffer);
    let script_str = WrapErr::wrap_err(script_str_res, "Failed to parse as string")?;
    let script_res = ron::from_str(script_str);
    let script: DocScript = WrapErr::wrap_err(script_res, "Failed to parse DocScript")?;

    println!("script: {:#?}", script);
    println!("opt: {:?}", opt);

    let doc_opt = Options {
        out: opt.out.clone(),
        with_images: None,
        print_driver: None,
        page: None,
        format: Format::PDF,
        cl_meta: Meta::default(),
        meta: None,
    };

    // Set-Up font cache
    let folder = file.parent().unwrap();
    let chsets_folder = folder.join(&script.chsets);
    let chsets_folder: PathBuf = chsets_folder.canonicalize().wrap_err_with(|| {
        format!(
            "Failed to canonicalize CHSETS folder path `{}`",
            chsets_folder.display()
        )
    })?;
    let mut fc = FontCache::new(chsets_folder);

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
        let mut document = Document::new(&doc_opt, doc_file);

        document.process_sdoc(&input, &mut fc)?;
        documents.push(document);
    }

    // Preprare output
    let mut hnd = Handle::new();

    prepare_meta(&mut hnd, &script.meta)?;

    let mut use_table_vec = UseTableVec::new();
    for doc in &documents {
        let use_matrix = doc.use_matrix();
        use_table_vec.append(&doc.chsets, use_matrix);
    }

    // FIXME: Auto-Detect from font cache
    let pk = PrinterKind::Laser30;

    let fonts_capacity = fc.chsets().len();
    let mut font_info = Fonts::new(fonts_capacity, hnd.res.fonts.len());

    for font in font_info.make_fonts(&fc, use_table_vec, pk)? {
        hnd.res.fonts.push(font);
    }

    for doc in &documents {
        prepare_document(&mut hnd, doc, &script.meta, &font_info, pk)?;
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

    hnd.outline.children = map_outline_items(&script.outline)?;

    handle_out(&opt.out, &file, hnd)?;
    Ok(())
}
