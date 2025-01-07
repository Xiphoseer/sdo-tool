use std::{collections::BTreeMap, fs::File, io::BufWriter, path::Path};

use color_eyre::eyre::{self, eyre, OptionExt};
use log::info;
use pdf_create::{
    common::{MediaBox, OutputIntent, OutputIntentSubtype, PdfString, ProcSet, Rectangle},
    high::{DictResource, Handle, Page, Resource, Resources, XObject},
};
use sdo_pdf::{
    font::Fonts,
    prepare_info,
    sdoc::{write_pdf_page, Contents},
    write_pdf_page_images, MetaInfo,
};
use signum::{
    chsets::{
        cache::{ChsetCache, FontCacheInfo},
        FontKind, UseTableVec,
    },
    docs::Overrides,
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

const FONTS: [&str; 8] = ["C0", "C1", "C2", "C3", "C4", "C5", "C6", "C7"];

pub fn prepare_document(
    hnd: &mut Handle,
    doc: &Document,
    di: &DocumentInfo,
    overrides: &Overrides,
    font_info: &Fonts,
) -> eyre::Result<()> {
    let print = &di.fonts;
    let mut fonts = BTreeMap::new();
    let mut infos = [None; 8];
    for (cset, info) in print
        .chsets
        .iter()
        .map(FontCacheInfo::index)
        .enumerate()
        .filter_map(|(cset, fc_index)| {
            fc_index
                .and_then(|fc_index| font_info.get(fc_index))
                .map(|info| (cset, info))
        })
    {
        fonts.insert(
            FONTS[cset].to_owned(),
            Resource::Global {
                index: font_info.index(info),
            },
        );
        infos[cset] = Some(info);
    }

    let font_dict = hnd.res.font_dicts.len();
    hnd.res.font_dicts.push(fonts);

    // PDF uses a unit length of 1/72 1/(18*4) of an inch by default
    //
    // Signum uses 1/54 1/(18*3) of an inch vertically and 1/90 1/(18*5) horizontally

    for page in &doc.tebu {
        let page_info = doc.pages[page.index as usize].as_ref().unwrap();
        let image_sites = &doc.sites[..];
        let res = &mut hnd.res;

        let media_box = MediaBox::A4;
        let mut contents = Contents::for_page(page_info, &media_box, overrides);

        let mut x_objects = DictResource::<XObject>::new();

        let has_images = write_pdf_page_images(
            &mut contents,
            di,
            page_info,
            image_sites,
            res,
            &mut x_objects,
        );

        let proc_sets = {
            let mut sets = vec![ProcSet::PDF, ProcSet::Text];
            if has_images {
                sets.push(ProcSet::ImageB);
            }
            sets
        };
        let resources = Resources {
            fonts: Resource::Global { index: font_dict },
            x_objects: Resource::Immediate(Box::new(x_objects)),
            proc_sets,
        };

        let mut contents = contents.start_text(1.0, -1.0);

        write_pdf_page(&mut contents, print, &infos, page)?;

        let contents = contents.into_inner();

        let page = Page {
            media_box: Rectangle::from(media_box),
            resources,
            contents,
        };
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
