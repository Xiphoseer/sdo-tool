use std::{
    borrow::Cow, collections::BTreeMap, fs::File, io::BufWriter, ops::Range, path::Path, usize,
};

use color_eyre::eyre::{self, eyre};
use log::{debug, info};
use pdf_create::{
    common::{
        ColorIs, ColorSpace, ICCColorProfileMetadata, ImageMetadata, LabColorSpaceParams,
        OutputIntent, OutputIntentSubtype, PdfString, Point, ProcSet, Rectangle,
    },
    high::{
        self, DictResource, Font, Handle, ICCBasedColorProfile, Image, Page, Resource,
        ResourceIndex, Resources, XObject,
    },
};
use regex::RegexSet;
use sdo_pdf::{
    font::{encode_byte, FontInfo, FontVariant, Fonts, Type3FontFamily},
    sdoc::Contents,
};
use signum::{
    chsets::{cache::ChsetCache, FontKind, UseTableVec},
    docs::tebu::{Line, PageText, Style},
};

use crate::cli::opt::{Destination, Meta, Options, OutlineItem};

use super::Document;

pub const TEX_SRBG_COLOR_PROFILE: &[u8] = include_bytes!("../../../res/sRGB.icc");
pub const ICC_SRBG_COLOR_PROFILE: &[u8] = include_bytes!("../../../res/sRGB_v4_ICC_preference.icc");
pub const ICC_SRBG_2014_COLOR_PROFILE: &[u8] = include_bytes!("../../../res/sRGB2014.icc");

pub fn prepare_meta(hnd: &mut Handle, meta: &Meta) -> eyre::Result<()> {
    // Metadata
    hnd.meta.author = meta.author.clone();
    if let Some(subject) = &meta.subject {
        hnd.meta.subject = Some(subject.clone());
    }
    if let Some(title) = &meta.title {
        hnd.meta.title = Some(title.clone());
    }
    hnd.meta.creator = Some("SIGNUM © 1986-93 F. Schmerbeck".to_string());
    hnd.meta.producer = "Signum! Document Toolbox".to_string();

    // Output intents
    hnd.output_intents.push(OutputIntent {
        subtype: OutputIntentSubtype::GTS_PDFA1,
        dest_output_profile: Some(ICCBasedColorProfile {
            stream: TEX_SRBG_COLOR_PROFILE,
            meta: ICCColorProfileMetadata {
                alternate: Some(ColorSpace::DeviceRGB),
                num_components: 3,
            },
        }),
        output_condition: None,
        output_condition_identifier: PdfString::new("IEC sRGB"),
        registry_name: Some(PdfString::new("http://www.iec.ch")),
        info: Some(PdfString::new(
            "IEC 61966-2.1 Default RGB colour space - sRGB",
        )),
    });

    Ok(())
}

const FONTS_REGULAR: [&str; 8] = ["C0", "C1", "C2", "C3", "C4", "C5", "C6", "C7"];
const FONTS_ITALIC: [&str; 8] = ["I0", "I1", "I2", "I3", "I4", "I5", "I6", "I7"];
const FONTS_BOLD: [&str; 8] = ["B0", "B1", "B2", "B3", "B4", "B5", "B6", "B7"];
const FONTS_BOLD_ITALIC: [&str; 8] = ["X0", "X1", "X2", "X3", "X4", "X5", "X6", "X7"];

struct PageContext<'a, 'b> {
    offset: Point<Option<i32>>,
    font_infos: [Option<&'b FontInfo>; 8],
    font_dict_resource_index: ResourceIndex<DictResource<Font<'a>>>,
}

fn prepare_page<'a>(
    hnd: &mut Handle<'a>,
    doc: &Document,
    index: usize,
    page: &PageText,
    ctx: &PageContext<'a, '_>,
    auto_outline: &mut AutoOutline,
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
                    color_space: ColorSpace::Lab(LabColorSpaceParams::default()),
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
        fonts: Resource::Global(ctx.font_dict_resource_index),
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

    let left = xmargin as f32 + ctx.offset.x.unwrap_or(0) as f32;
    let left = left - page_info.format.left as f32 * 8.0 / 10.0;
    let top = ymargin as f32 + ctx.offset.y.unwrap_or(0) as f32;
    let top = a4_height as f32 - top - 8.0;
    let media_box = Rectangle::media_box(a4_width, a4_height);

    let mut contents = Contents::new(top, left);

    for (site, key) in img {
        contents.image(site, &key).unwrap();
    }

    const FONT_SIZE: i32 = 10;
    const FONTUNITS_PER_SIGNUM_X: i32 = 800 / FONT_SIZE;

    draw_underlines(doc, &ctx.font_infos, &page.content, &mut contents)?;

    let mut contents = contents.start_text(1.0, -1.0);

    for (line_index, (skip, line)) in page.content.iter().enumerate() {
        let first_style = line.data.first().map(|x| x.style).unwrap_or_default();
        let mut is_same_style = !auto_outline.req_same_style || first_style != Style::default();

        contents.next_line(0, *skip as u32 + 1);

        let mut prev_width = 0;
        let mut text = String::new();

        for te in &line.data {
            let x = te.offset as i32;

            if is_same_style {
                is_same_style &= te.style == first_style;
            }

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

                if !text.is_empty() {
                    text.push(' ');
                }
            }

            let win_ansi_byte = encode_byte(te.cval);
            contents.byte(win_ansi_byte)?;

            let csu = te.cset as usize;
            let fi = ctx.font_infos[csu].ok_or_else(|| {
                let font_name = doc.cset[csu].as_deref().unwrap_or("");
                eyre!("Missing font #{}: {:?}", csu, font_name)
            })?;

            // Push the character
            text.push(fi.mappings().decode(te.cval));

            prev_width = fi.width(te.cval) as i32;
            if is_wide {
                prev_width *= 2;
            }
        }

        let is_all_box = text.chars().all(|x| matches!(x, '|' | '_' | ' ' | '.'));
        let is_line_index_ok = line_index >= auto_outline.min_line_index;
        //let is_not_align = !line.flags.contains(Flags::ALIG);
        //let is_para = line.flags.contains(Flags::PARA);
        let style_ok = !auto_outline.req_same_style || is_same_style;

        let matches = auto_outline.title_set.matches(&text);
        if is_line_index_ok && style_ok && !is_all_box && matches.matched_any() {
            let mut level = matches.iter().next().unwrap();

            // Check auto outline
            if auto_outline.in_toc {
                let auto_toc = auto_outline.toc.as_ref().unwrap();
                log::info!("Check page {} against {:?}", index, auto_toc.page_range);
                if auto_toc.page_range.contains(&index) {
                    level += 1;
                } else {
                    auto_outline.in_toc = false;
                }
            }

            let out = recurse_outline_level(level, &mut auto_outline.items);

            // Trim trailing colon
            if text.ends_with(':') {
                text.pop();
            }

            if let Some(auto_toc) = auto_outline.toc.as_ref() {
                log::info!("Found {} on page {}", text, index + 1);

                if !auto_outline.in_toc && text == auto_toc.title {
                    auto_outline.in_toc = true;
                    log::info!("Found TOC!");
                };
            }

            let y_pos = top - contents.get_y() + 50.0;
            out.push(OutlineItem {
                title: text,
                dest: Destination::PageFitH(index, y_pos as usize),
                children: vec![],
            });
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

fn recurse_outline_level(level: usize, o: &mut Vec<OutlineItem>) -> &mut Vec<OutlineItem> {
    if level == 0 || o.is_empty() {
        return o;
    }
    let inner = o.last_mut().unwrap();
    recurse_outline_level(level - 1, &mut inner.children)
}

fn draw_underlines(
    doc: &Document,
    font_infos: &[Option<&FontInfo>; 8],
    content: &[(u16, Line)],
    contents: &mut Contents,
) -> color_eyre::Result<()> {
    let mut y = 0;

    const UNITS_PER_SIGNUM_X: f32 = 0.8;

    // Draw underlines
    for (skip, line) in content {
        y += *skip as u32 + 1;

        let mut underline_start = None;

        let mut prev_width = 0.0;
        let mut x = 0.0;

        for te in &line.data {
            let x_step = te.offset as i32;
            let x_step_pdf = x_step as f32 * UNITS_PER_SIGNUM_X;
            let x_new = x + x_step_pdf;

            let is_wide = te.style.wide;
            let is_underlined = te.style.underlined;

            // check underlined
            match (is_underlined, underline_start) {
                (true, None) => {
                    underline_start = Some(x_new);
                }
                (true, Some(_)) => { /* keep the start */ }
                (false, None) => { /* no underline */ }
                (false, Some(x_start)) => {
                    // underline ended after the previous char
                    let y_pos = y + 2;
                    let x_end = x + prev_width;
                    contents.draw_line(&[(x_start, y_pos), (x_end, y_pos)])?;
                    underline_start = None;
                }
            }

            // Find character
            let csu = te.cset as usize;
            let fi = font_infos[csu].ok_or_else(|| {
                let font_name = doc.cset[csu].as_deref().unwrap_or("");
                eyre!("Missing font #{}: {:?}", csu, font_name)
            })?;

            // Update variables
            x = x_new;
            // div by 1000 (font matrix) mul by 10 (font size)
            prev_width = fi.width(te.cval) as f32 / 100.0;
            if is_wide {
                prev_width *= 2.0;
            }
        }

        // Finish underlining the last char
        if let Some(x_start) = underline_start {
            let x_end = x + prev_width;
            let y_pos = y + 2;
            contents.draw_line(&[(x_start, y_pos), (x_end, y_pos)])?;
        }
    }
    Ok(())
}

struct AutoTOC {
    title: String,
    page_range: Range<usize>,
}

pub struct AutoOutline {
    items: Vec<OutlineItem>,
    title_set: RegexSet,
    min_line_index: usize,
    req_same_style: bool,
    in_toc: bool,
    toc: Option<AutoTOC>,
}

impl AutoOutline {
    /// Create a new auto outline
    pub fn new<S: AsRef<str>>(arg: &[S], min_line_index: usize) -> Result<Self, regex::Error> {
        Ok(Self {
            items: vec![],
            title_set: RegexSet::new(arg)?,
            min_line_index,
            in_toc: false,
            req_same_style: false,
            toc: None,
        })
    }

    pub fn req_same_style(&mut self, f: bool) {
        self.req_same_style = f;
    }

    pub fn set_auto_toc(&mut self, title: &str, page_range: Range<usize>) {
        self.toc = Some(AutoTOC {
            title: title.to_owned(),
            page_range,
        })
    }

    pub fn get_items(&self) -> &[OutlineItem] {
        &self.items
    }
}

pub fn prepare_document<'a>(
    hnd: &mut Handle<'a>,
    doc: &Document,
    page_index_offset: usize,
    meta: &Meta,
    font_info: &Fonts,
    auto_outline: &mut AutoOutline,
) -> eyre::Result<()> {
    let (fonts, font_infos) = find_fonts(doc, font_info);
    let font_dict_resource_index = hnd.res.push_font_dict(fonts);

    // PDF uses a unit length of 1/72 1/(18*4) of an inch by default
    //
    // Signum uses 1/54 1/(18*3) of an inch vertically and 1/90 1/(18*5) horizontally

    let ctx = PageContext {
        offset: Point {
            x: meta.xoffset,
            y: meta.yoffset,
        },
        font_infos,
        font_dict_resource_index,
    };

    for (index, page) in doc.tebu.iter().enumerate() {
        let page_index = page_index_offset + index;
        let page = prepare_page(hnd, doc, page_index, page, &ctx, auto_outline)?;
        hnd.pages.push(page);
    }

    Ok(())
}

fn find_fonts<'a, 'b>(
    doc: &Document,
    font_info: &'b Fonts,
) -> (
    BTreeMap<String, Resource<Font<'a>>>,
    [Option<&'b FontInfo>; 8],
) {
    let mut fonts = BTreeMap::new();
    const INDEX: [usize; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
    let enumerated = INDEX.map(|cset| (cset, doc.chsets[cset]));

    let font_infos = enumerated.map(|(cset, fc_index)| {
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

                return Some(info);
            }
        }
        None
    });
    (fonts, font_infos)
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
    auto_outline: &mut AutoOutline,
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

    prepare_document(&mut hnd, doc, 0, &meta, &font_info, auto_outline)?;
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
    let mut auto_outline = AutoOutline::new(&[] as &[&str], 0)?;
    let hnd = process_doc(doc, opt, fc, &mut auto_outline)?;
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
