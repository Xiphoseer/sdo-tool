use std::{collections::BTreeMap, fs::File, io::BufWriter, path::Path};

use color_eyre::eyre::{self, eyre, OptionExt};
use log::{debug, info};
use pdf_create::{
    common::{OutputIntent, OutputIntentSubtype, PdfString, ProcSet, Rectangle},
    high::{DictResource, Handle, Page, Resource, Resources, XObject},
};
use sdo_pdf::{font::Fonts, image_for_site, prepare_info, sdoc::Contents, MetaInfo};
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
    meta: &Overrides,
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

        let mut x_objects: DictResource<XObject> = BTreeMap::new();
        let mut img = vec![];
        let image_sites = &doc.sites[..];
        for (index, site) in image_sites
            .iter()
            .enumerate()
            .filter(|(_, site)| site.page == page_info.phys_pnr)
        {
            let key = format!("I{}", index);
            debug!(
                "Adding image from #{} on page {} as /{}",
                site.img, page_info.log_pnr, &key
            );

            let image = image_for_site(di, site);

            x_objects.insert(key.clone(), hnd.res.push_xobject(image));
            img.push((site, key));
        }

        let mut proc_sets = vec![ProcSet::PDF, ProcSet::Text];
        if !img.is_empty() {
            proc_sets.push(ProcSet::ImageB);
        }
        let resources = Resources {
            fonts: Resource::Global { index: font_dict },
            x_objects: Resource::Immediate(Box::new(x_objects)),
            proc_sets,
        };

        let a4_width = 592;
        let a4_height = 842;

        let width = page_info.format.width() * 72 / 90;
        let height = page_info.format.length as i32 * 72 / 54;

        assert!(width as i32 <= a4_width, "Please file a bug!");

        let xmargin = (a4_width - width as i32) / 2;
        let ymargin = (a4_height - height) / 2;

        let left = xmargin as f32 + meta.xoffset as f32;
        let left = left - page_info.format.left as f32 * 8.0 / 10.0;
        let top = ymargin as f32 + meta.yoffset as f32;
        let top = a4_height as f32 - top - 8.0;
        let media_box = Rectangle::media_box(a4_width, a4_height);

        let mut contents = Contents::new(top, left);

        for (site, key) in img {
            contents.image(site, &key).unwrap();
        }

        let mut contents = contents.start_text(1.0, -1.0);

        for (skip, line) in &page.content {
            contents.next_line(0, *skip as u32 + 1);

            const FONTUNITS_PER_SIGNUM_X: i32 = 800;
            let mut prev_width = 0;
            for te in &line.data {
                let x = te.offset as i32;

                let is_wide = te.style.wide;
                let is_tall = te.style.tall;

                let font_size = if is_tall { 2 } else { 1 };
                let font_width = match (is_tall, is_wide) {
                    (true, true) => 100,
                    (true, false) => 50,
                    (false, true) => 200,
                    (false, false) => 100,
                };

                contents.cset(te.cset, font_size);
                contents.fwidth(font_width);

                let mut diff = x * FONTUNITS_PER_SIGNUM_X - prev_width;
                if diff != 0 {
                    if is_wide {
                        diff /= 2;
                    }
                    contents.xoff(-diff);
                }
                contents.byte(te.cval);

                let csu = te.cset as usize;
                let fi = infos[csu].ok_or_else(|| {
                    let font_name = print.chsets[csu].name().unwrap_or("");
                    eyre!("Missing font #{}: {:?}", csu, font_name)
                })?;
                prev_width = fi.width(te.cval) as i32;
                if is_wide {
                    prev_width *= 2;
                }
            }

            contents.flush();
        }

        let contents = contents.into_inner();

        let page = Page {
            media_box,
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
