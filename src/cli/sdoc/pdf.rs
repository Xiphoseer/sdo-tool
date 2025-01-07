use std::{fs::File, io::BufWriter, path::Path};

use color_eyre::eyre::{self, eyre, OptionExt};
use log::info;
use pdf_create::{
    common::{OutputIntent, OutputIntentSubtype, PdfString},
    high::Handle,
};
use sdo_pdf::{
    font::{font_dict, Fonts},
    prepare_info,
    sdoc::generate_pdf_page,
    MetaInfo,
};
use signum::{
    chsets::{cache::ChsetCache, FontKind, UseTableVec},
    docs::{hcim::ImageSite, GenerationContext, Overrides},
};

use super::{Document, DocumentInfo};

pub fn prepare_meta(hnd: &mut Handle, meta: &MetaInfo) -> eyre::Result<()> {
    prepare_info(&mut hnd.info, meta)?;

    // Output intents
    hnd.output_intents.push(OutputIntent {
        subtype: OutputIntentSubtype::GTS_PDFA1,
        output_condition: None,
        output_condition_identifier: PdfString::new("FOO"),
        registry_name: None,
        info: None,
    });

    Ok(())
}

struct GenCtx<'a> {
    di: &'a DocumentInfo,
    image_sites: &'a [ImageSite],
}

impl GenerationContext for GenCtx<'_> {
    fn image_sites(&self) -> &[ImageSite] {
        self.image_sites
    }

    fn document_info(&self) -> &DocumentInfo {
        self.di
    }
}

pub fn prepare_document(
    hnd: &mut Handle,
    doc: &Document,
    di: &DocumentInfo,
    overrides: &Overrides,
    font_info: &Fonts,
) -> eyre::Result<()> {
    let gc = GenCtx {
        di,
        image_sites: &doc.sites[..],
    };

    let (fonts, infos) = font_dict(font_info, &di.fonts);

    let font_dict = hnd.res.push_font_dict(fonts);

    for page in &doc.tebu {
        let page_info = doc.pages[page.index as usize].as_ref().unwrap();
        let res = &mut hnd.res;

        let page = generate_pdf_page(
            &gc,
            overrides,
            &infos,
            font_dict.clone(),
            page,
            page_info,
            res,
        )?;
        hnd.pages.push(page);
    }

    Ok(())
}

fn doc_meta(doc: &Document) -> eyre::Result<(MetaInfo, Overrides)> {
    let meta = doc.opt.meta()?;
    let file_name = doc
        .opt
        .file
        .file_name()
        .ok_or_eyre("expect file to have name")?;
    let file_name = file_name
        .to_str()
        .ok_or_eyre("File name contains invalid characters")?;
    let info = meta.pdf_meta_info(file_name);
    let overrides = meta.to_overrides();
    Ok((info, overrides))
}

pub fn process_doc<'a>(
    doc: &'a Document,
    fc: &'a ChsetCache,
    di: &DocumentInfo,
    pd: Option<FontKind>,
) -> eyre::Result<Handle<'a>> {
    let mut hnd = Handle::new();

    let (meta, overrides) = doc_meta(doc)?;
    prepare_meta(&mut hnd, &meta)?;

    let use_matrix = doc.use_matrix();
    let mut use_table_vec = UseTableVec::new();
    use_table_vec.append(&di.fonts.chsets, use_matrix);

    let pd = pd.ok_or_else(|| eyre!("No printer type selected"))?;

    let pk = if let FontKind::Printer(pk) = pd {
        pk
    } else {
        return Err(eyre!("Editor fonts are not currently supported"));
    };

    let mut font_info = Fonts::new(8, 0); // base = hnd.res.fonts.len() ???

    for font in font_info.make_fonts(fc, use_table_vec, pk) {
        hnd.res.fonts.push(font);
    }

    prepare_document(&mut hnd, doc, di, &overrides, &font_info)?;
    Ok(hnd)
}

pub fn output_pdf(
    doc: &Document,
    fc: &ChsetCache,
    di: &DocumentInfo,
    pd: Option<FontKind>,
) -> eyre::Result<()> {
    let hnd = process_doc(doc, fc, di, pd)?;
    handle_out(doc.opt.out.as_deref(), &doc.opt.file, hnd)?;
    Ok(())
}

pub fn handle_out(out: Option<&Path>, file: &Path, hnd: Handle) -> eyre::Result<()> {
    if out == Some(Path::new("-")) {
        println!("----------------------------- PDF -----------------------------");
        let stdout = std::io::stdout();
        let mut stdolock = stdout.lock();
        hnd.write(&mut stdolock)?;
        println!("---------------------------------------------------------------");
        Ok(())
    } else {
        let out = out.unwrap_or_else(|| file.parent().unwrap());
        let file = file.file_stem().unwrap();
        let out = {
            let mut buf = out.join(file);
            buf.set_extension("pdf");
            buf
        };
        let out_file = File::create(&out)?;
        let mut out_buf = BufWriter::new(out_file);
        info!("Writing `{}` ...", out.display());
        hnd.write(&mut out_buf)?;
        info!("Done!");
        Ok(())
    }
}
