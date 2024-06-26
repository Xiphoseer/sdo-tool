use std::{borrow::Cow, collections::BTreeMap, fs::File, io::BufWriter, path::Path, usize};

use color_eyre::eyre::{self, eyre};
use log::{debug, info};
use pdf_create::{
    chrono::Local,
    common::{
        ColorIs, ColorSpace, ImageMetadata, OutputIntent, OutputIntentSubtype, PdfString, ProcSet,
        Rectangle,
    },
    encoding::pdf_doc_encode,
    high::{DictResource, Handle, Image, Page, Resource, Resources, XObject},
};
use sdo_pdf::{font::Fonts, sdoc::Contents};
use signum::chsets::{
    cache::{ChsetCache, FontCacheInfo},
    FontKind, UseTableVec,
};

use crate::cli::opt::Meta;

use super::{Document, DocumentInfo};

pub fn prepare_meta(hnd: &mut Handle, meta: &Meta) -> eyre::Result<()> {
    // Metadata
    if let Some(author) = &meta.author {
        let author = pdf_doc_encode(author)?;
        hnd.info.author = Some(PdfString::new(author));
    }
    if let Some(subject) = &meta.subject {
        let subject = pdf_doc_encode(subject)?;
        hnd.info.subject = Some(PdfString::new(subject));
    }
    if let Some(title) = &meta.title {
        let title = pdf_doc_encode(title)?;
        hnd.info.title = Some(PdfString::new(title));
    }
    let creator = pdf_doc_encode("SIGNUM © 1986-93 F. Schmerbeck")?;
    hnd.info.creator = Some(PdfString::new(creator));
    let producer = pdf_doc_encode("Signum! Document Toolbox")?;
    hnd.info.producer = Some(PdfString::new(producer));

    let now = Local::now();
    hnd.info.creation_date = Some(now);
    hnd.info.mod_date = Some(now);

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
    meta: &Meta,
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
        for (index, site) in doc.sites.iter().enumerate() {
            if site.page == page_info.phys_pnr {
                let key = format!("I{}", index);
                let width = site.sel.w as usize;
                let height = site.sel.h as usize;
                //let area = width * height;

                let img_num = site.img as usize;
                let (_, im) = &di.images[img_num];
                let data = im.select(site.sel);

                let img_index = hnd.res.x_objects.len();
                hnd.res.x_objects.push(XObject::Image(Image {
                    meta: ImageMetadata {
                        width,
                        height,
                        color_space: ColorSpace::DeviceGray,
                        bits_per_component: 1,
                        image_mask: true,
                        decode: ColorIs::One,
                    },
                    data,
                }));
                debug!(
                    "Adding image from #{} on page {} as /{}",
                    img_num, page_info.log_pnr, &key
                );
                x_objects.insert(key.clone(), Resource::Global { index: img_index });
                img.push((site, key));
            }
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

        let left = xmargin as f32 + meta.xoffset.unwrap_or(0) as f32;
        let left = left - page_info.format.left as f32 * 8.0 / 10.0;
        let top = ymargin as f32 + meta.yoffset.unwrap_or(0) as f32;
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

fn doc_meta<'a>(doc: &'a Document) -> eyre::Result<Cow<'a, Meta>> {
    let meta = doc.opt.meta()?;
    if meta.title.is_none() {
        let mut meta = meta.into_owned();
        let file_name = doc.opt.file.file_name().unwrap();
        let title = file_name
            .to_str()
            .ok_or_else(|| eyre!("File name contains invalid characters"))?;
        meta.title = Some(title.to_owned());
        Ok(Cow::Owned(meta))
    } else {
        Ok(meta)
    }
}

pub fn process_doc<'a>(
    doc: &'a Document,
    fc: &'a ChsetCache,
    di: &DocumentInfo,
    pd: Option<FontKind>,
) -> eyre::Result<Handle<'a>> {
    let mut hnd = Handle::new();

    let meta = doc_meta(doc)?;
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

    prepare_document(&mut hnd, doc, di, &meta, &font_info)?;
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
