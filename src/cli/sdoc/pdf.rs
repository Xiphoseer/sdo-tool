use std::{borrow::Cow, collections::BTreeMap, fs::File, io::BufWriter, path::Path};

use color_eyre::eyre::{self, eyre};
use log::{debug, info};
use pdf_create::{
    chrono::Local,
    common::{
        ColorIs, ColorSpace, ImageMetadata, OutputIntent, OutputIntentSubtype, PdfString, Point,
        ProcSet, Rectangle,
    },
    encoding::pdf_doc_encode,
    high::{
        self, DictResource, Font, Handle, Image, Page, Resource, ResourceIndex, Resources, XObject,
    },
};
use sdo_pdf::{
    font::{FontInfo, FontVariant, Fonts, Type3FontFamily},
    sdoc::Contents,
};
use signum::{
    chsets::{cache::ChsetCache, FontKind, UseTableVec},
    docs::tebu::PageText,
};

use crate::cli::opt::{Meta, Options};

use super::Document;

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

const FONTS_REGULAR: [&str; 8] = ["C0", "C1", "C2", "C3", "C4", "C5", "C6", "C7"];
const FONTS_ITALIC: [&str; 8] = ["I0", "I1", "I2", "I3", "I4", "I5", "I6", "I7"];
const FONTS_BOLD: [&str; 8] = ["B0", "B1", "B2", "B3", "B4", "B5", "B6", "B7"];
const FONTS_BOLD_ITALIC: [&str; 8] = ["X0", "X1", "X2", "X3", "X4", "X5", "X6", "X7"];

pub fn prepare_document(
    hnd: &mut Handle,
    doc: &Document,
    meta: &Meta,
    font_info: &Fonts,
) -> eyre::Result<()> {
    let mut fonts = BTreeMap::new();
    let mut font_infos = [None; 8];

    for (cset, fc_index) in doc.chsets.iter().copied().enumerate() {
        if let Some(fc_index) = fc_index {
            let key_regular = FONTS_REGULAR[cset].to_owned();
            let key_italic = FONTS_ITALIC[cset].to_owned();
            let key_bold = FONTS_BOLD[cset].to_owned();
            let key_bold_italic = FONTS_BOLD_ITALIC[cset].to_owned();
            if let Some(info) = font_info.get(fc_index) {
                let index_regular = font_info.index(info, FontVariant::Regular);
                let index_italic = font_info.index(info, FontVariant::Italic);
                let index_bold = font_info.index(info, FontVariant::Bold);
                let index_bold_italic = font_info.index(info, FontVariant::BoldItalic);

                fonts.insert(key_regular, Resource::Global(index_regular));
                fonts.insert(key_italic, Resource::Global(index_italic));
                fonts.insert(key_bold, Resource::Global(index_bold));
                fonts.insert(key_bold_italic, Resource::Global(index_bold_italic));

                font_infos[cset] = Some(info);
            }
        }
    }

    let font_dict_resource_index = hnd.res.push_font_dict(fonts);

    // PDF uses a unit length of 1/72 1/(18*4) of an inch by default
    //
    // Signum uses 1/54 1/(18*3) of an inch vertically and 1/90 1/(18*5) horizontally

    let offset = Point {
        x: meta.xoffset.unwrap_or(0),
        y: meta.yoffset.unwrap_or(0),
    };
    for page in doc.tebu.iter() {
        let page = prepare_page(hnd, doc, &offset, page, font_infos, font_dict_resource_index)?;
        hnd.pages.push(page);
    }

    Ok(())
}

pub fn prepare_page<'a>(
    hnd: &mut Handle<'a>,
    doc: &Document,
    offset: &Point<i32>,
    page: &PageText,
    font_infos: [Option<&FontInfo>; 8],
    font_dict_resource_index: ResourceIndex<DictResource<Font<'a>>>,
) -> eyre::Result<Page<'a>> {
    let page_info = doc.pages[page.index as usize].as_ref().unwrap();

    let mut x_objects: DictResource<XObject> = BTreeMap::new();
    let mut img = vec![];
    for (index, site) in doc.sites.iter().enumerate() {
        if site.page == page_info.phys_pnr {
            let key = format!("I{}", index);
            let width = site.sel.w as usize;
            let height = site.sel.h as usize;

            let img_num = site.img as usize;
            let im = &doc.images[img_num].image;
            let data = im.select(site.sel);

            let res_index = hnd.res.push_x_object(XObject::Image(Image {
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
            x_objects.insert(key.clone(), Resource::Global(res_index));
            img.push((site, key));
        }
    }

    let mut proc_sets = vec![ProcSet::PDF, ProcSet::Text];
    if !img.is_empty() {
        proc_sets.push(ProcSet::ImageB);
    }
    let resources = Resources {
        fonts: Resource::Global(font_dict_resource_index),
        x_objects: Resource::Immediate(Box::new(x_objects)),
        proc_sets,
    };

    let a4_width = 592;
    let a4_height = 842;

    let width = page_info.format.width() * 72 / 90;
    let height = page_info.format.length as i32 * 72 / 54;

    assert!(width as i32 <= a4_width, "Please file a bug!");

    let xmargin = (a4_width - width as i32) / 2;
    let ymargin = (a4_height - height as i32) / 2;

    let left = xmargin as f32 + offset.x as f32;
    let left = left - page_info.format.left as f32 * 8.0 / 10.0;
    let top = ymargin as f32 + offset.y as f32;
    let top = a4_height as f32 - top - 8.0;
    let media_box = Rectangle::media_box(a4_width, a4_height);

    let mut contents = Contents::new(top, left);

    for (site, key) in img {
        contents.image(site, &key).unwrap();
    }

    let mut contents = contents.start_text(1.0, -1.0);

    const FONT_SIZE: i32 = 10;
    const FONTUNITS_PER_SIGNUM_X: i32 = 800 / FONT_SIZE;

    for (skip, line) in &page.content {
        contents.next_line(0, *skip as u32 + 1);

        let mut prev_width = 0;
        for te in &line.data {
            let x = te.offset as i32;

            let is_wide = te.style.wide;
            let is_tall = te.style.tall;

            let font_size = if is_tall { 20 } else { 10 };
            let font_width = match (is_tall, is_wide) {
                (true, true) => 100,
                (true, false) => 50,
                (false, true) => 200,
                (false, false) => 100,
            };

            let font_variant = match (te.style.italic, te.style.bold) {
                (true, true) => FontVariant::BoldItalic,
                (true, false) => FontVariant::Italic,
                (false, true) => FontVariant::Bold,
                (false, false) => FontVariant::Regular,
            };

            contents.cset(te.cset, font_size, font_variant);
            contents.fwidth(font_width);

            let mut diff = x * FONTUNITS_PER_SIGNUM_X - prev_width;
            if diff != 0 {
                if is_wide {
                    diff /= 2;
                }
                contents.xoff(-diff)?;
            }

            contents.byte(te.cval)?;

            let csu = te.cset as usize;
            let fi = font_infos[csu].ok_or_else(|| {
                let font_name = doc.cset[csu].as_deref().unwrap_or("");
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

    Ok(Page {
        media_box,
        resources,
        contents,
    })
}

fn doc_meta(opt: &Options) -> eyre::Result<Cow<Meta>> {
    let meta = opt.meta()?;
    if meta.title.is_none() {
        let mut meta = meta.into_owned();
        let file_name = opt.file.file_name().unwrap();
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
    opt: &'a Options,
    fc: &'a ChsetCache,
) -> eyre::Result<Handle<'a>> {
    let mut hnd = Handle::new();

    let meta = doc_meta(opt)?;
    prepare_meta(&mut hnd, &meta)?;

    let use_matrix = doc.use_matrix();
    let mut use_table_vec = UseTableVec::new();
    use_table_vec.append(&doc.chsets, use_matrix);

    let pd = fc.print_driver(opt.print_driver)?;

    let pk = if let FontKind::Printer(pk) = pd {
        pk
    } else {
        return Err(eyre!("Editor fonts are not currently supported"));
    };

    let mut font_info = Fonts::new(8, 0); // base = hnd.res.fonts.len() ???

    let fonts = font_info.make_fonts(fc, use_table_vec, pk);
    push_fonts(&mut hnd, fonts);

    prepare_document(&mut hnd, doc, &opt.cl_meta, &font_info)?;
    Ok(hnd)
}

const VARIANTS: [FontVariant; 4] = [
    FontVariant::Regular,
    FontVariant::Italic,
    FontVariant::Bold,
    FontVariant::BoldItalic,
];

pub fn push_fonts<'a>(hnd: &mut Handle<'a>, font_families: Vec<Type3FontFamily<'a>>) {
    for (_index, family) in font_families.into_iter().enumerate() {
        let char_procs = hnd.res.push_char_procs(family.char_procs);
        let char_procs_bold = hnd.res.push_char_procs(family.bold_char_procs);
        let encoding = hnd.res.push_encoding(family.encoding);

        for key in VARIANTS {
            let var = family.font_variants.get(&key).unwrap();
            let char_procs = if matches!(key, FontVariant::Bold | FontVariant::BoldItalic) {
                high::Resource::Global(char_procs_bold)
            } else {
                high::Resource::Global(char_procs)
            };
            hnd.res.fonts.push(high::Font::Type3(high::Type3Font {
                name: Some(var.name.clone()),
                font_matrix: var.font_matrix,
                font_descriptor: Some(var.font_descriptor.clone()),
                font_bbox: family.font_bbox,
                first_char: family.first_char,
                last_char: family.last_char,
                char_procs,
                encoding: high::Resource::Global(encoding),
                widths: family.widths.clone(),
                to_unicode: family.to_unicode.clone(),
            }));
        }
    }
}

pub fn output_pdf(doc: &Document, opt: &Options, fc: &ChsetCache) -> eyre::Result<()> {
    let hnd = process_doc(doc, opt, fc)?;
    handle_out(opt.out.as_deref(), &opt.file, hnd)?;
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
