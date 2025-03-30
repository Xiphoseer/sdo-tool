//! # Signum! Documents

use std::io;

use log::warn;
use pdf_create::{
    common::{MediaBox, ProcSet, Rectangle},
    high::{DictResource, Font, GlobalResource, Handle, Page, Res, Resource, Resources, XObject},
};
use signum::{
    chsets::cache::{DocumentFontCacheInfo, FontCacheInfo},
    docs::{
        pbuf,
        tebu::{self, PageText},
        GenerationContext, Overrides,
    },
};

mod contents;
mod text;
use contents::Contents;
use text::{TextContents, TEXT_MATRIX_SCALE_X, TEXT_MATRIX_SCALE_Y};

use crate::{
    font::{FontInfo, Fonts, DEFAULT_FONT_SIZE, FONTUNITS_PER_SIGNUM_X},
    image::image_for_site,
    Error,
};

/// Write the text for a PDF page
fn write_pdf_page_text<O: io::Write>(
    contents: &mut TextContents<O>,
    print: &DocumentFontCacheInfo,
    infos: &[Option<&FontInfo>; 8],
    page: &PageText,
) -> Result<(), Error> {
    contents.goto_origin().map_err(Error::Contents)?;
    for (skip, line) in &page.content {
        contents.next_line(0, *skip as u32 + 1);

        // How far we've drawn
        let mut pdf_page_cursor: u32 = 0;

        for (cx, te) in line.characters() {
            let is_wide = te.style.is_wide();
            let is_tall = te.style.is_tall();
            let is_small = te.style.is_small();

            let default_font_size = DEFAULT_FONT_SIZE as u8;
            let default_font_width = 100.0f32;
            let (font_size, font_width) = if is_tall {
                (default_font_size * 3 / 2, default_font_width / 1.5) // * 1,5
            } else if is_small {
                (default_font_size * 3 / 4, default_font_width / 0.75) // * 0,75
            } else {
                (default_font_size, default_font_width)
            };

            let font_width = match is_wide {
                true => font_width * 2.0,
                false => font_width,
            };

            let csu = te.cset as usize;
            let fi = infos[csu].ok_or_else(|| {
                let font_name = print
                    .font_cache_info_at(csu)
                    .and_then(FontCacheInfo::name)
                    .unwrap_or("");
                Error::MissingFont(csu, font_name.to_owned())
            })?;
            let raw_width = fi.width(te.cval);
            let width = match is_wide {
                true => raw_width * 2,
                false => raw_width,
            };

            let is_bold = te.style.is_bold();
            contents
                .cset(te.cset, is_bold, font_size)
                .map_err(Error::Contents)?;
            contents.fwidth(font_width).map_err(Error::Contents)?;

            let cx_pdf = cx as u32 * (FONTUNITS_PER_SIGNUM_X / DEFAULT_FONT_SIZE as u32);
            let diff = cx_pdf.saturating_sub(pdf_page_cursor);
            if diff != 0 {
                let xoff = -(diff as i32 * DEFAULT_FONT_SIZE);
                contents.xoff(xoff).map_err(Error::Contents)?;
            }
            pdf_page_cursor += diff;

            // Note: slant has to be _after_ x-offset adjustment
            contents
                .slant(te.style.is_italic())
                .map_err(Error::Contents)?;
            let byte_width = width * DEFAULT_FONT_SIZE as u32;
            contents
                .byte(te.cval, byte_width)
                .map_err(Error::Contents)?;
            pdf_page_cursor += width;
        }

        contents.flush().map_err(Error::Contents)?;
    }
    Ok(())
}

fn write_pdf_page_underlines(
    print: &DocumentFontCacheInfo,
    font_infos: &[Option<&FontInfo>; 8],
    content: &[(u16, tebu::Line)],
    contents: &mut Contents,
) -> Result<(), Error> {
    let mut y = 0;

    const UNITS_PER_SIGNUM_X: f32 = 0.8;

    // Draw underlines
    for (skip, line) in content {
        y += *skip as u32 + 1;

        let mut underline_start = None;

        let mut prev_width = 0.0;
        let mut x = 0.0;

        for (cx, te) in line.characters() {
            let x_new = cx as f32 * UNITS_PER_SIGNUM_X;

            let is_wide = te.style.is_wide();
            let is_underlined = te.style.is_underlined();

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
                    contents
                        .draw_line(&[(x_start, y_pos), (x_end, y_pos)])
                        .map_err(Error::Contents)?;
                    underline_start = None;
                }
            }

            // Find character
            let csu = te.cset as usize;
            let fi = font_infos[csu].ok_or_else(|| {
                let font_name = print
                    .font_cache_info_at(csu)
                    .and_then(FontCacheInfo::name)
                    .unwrap_or("");
                Error::MissingFont(csu, font_name.to_owned())
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
            contents
                .draw_line(&[(x_start, y_pos), (x_end, y_pos)])
                .map_err(Error::Contents)?;
        }
    }
    Ok(())
}

/// Write the images of a PDF page
fn write_pdf_page_images<GC: GenerationContext>(
    contents: &mut Contents,
    gc: &GC,
    page_info: &pbuf::Page,
    res: &mut Res<'_>,
    x_objects: &mut DictResource<XObject>,
) -> bool {
    let mut has_images = false;
    for (index, site) in gc
        .image_sites()
        .iter()
        .enumerate()
        .filter(|(_, site)| site.page == page_info.phys_pnr)
    {
        let key = format!("I{}", index);
        log::debug!(
            "Adding image from #{} on page {} as /{}",
            site.img,
            page_info.log_pnr,
            &key
        );

        if let Some(image) = image_for_site(gc.document_info(), site) {
            contents.image(site, &key).unwrap();
            x_objects.insert(key.clone(), res.push_xobject(image).into());
            has_images |= true;
        } else {
            warn!("Missing image {} on page {}", site.img, site.page);
        }
    }
    has_images
}

/// Select a suitable media box
fn select_media_box(page_info: &pbuf::Page) -> MediaBox {
    let page_format = &page_info.format;
    let width = page_format.width() as i32 * 72 / 90;
    let height = page_format.length as i32 * 72 / 54;
    if width <= MediaBox::A4.width && height <= MediaBox::A4.height {
        MediaBox::A4
    } else if width <= MediaBox::A4_LANDSCAPE.width && height <= MediaBox::A4_LANDSCAPE.height {
        MediaBox::A4_LANDSCAPE
    } else {
        MediaBox { width, height }
    }
}

/// Generate a single PDF page
pub fn generate_pdf_page<GC: GenerationContext>(
    gc: &GC,
    overrides: &Overrides,
    infos: &[Option<&FontInfo>; 8],
    fonts: GlobalResource<DictResource<Font<'static>>>,
    page: &tebu::PageText,
    page_info: &pbuf::Page,
    res: &mut Res<'_>,
) -> Result<Page<'static>, Error> {
    let media_box = select_media_box(page_info);
    let mut x_objects = DictResource::<XObject>::new();

    let has_images: bool;
    let contents = {
        let mut contents = Contents::for_page(page_info, &media_box, overrides);
        has_images = write_pdf_page_images(&mut contents, gc, page_info, res, &mut x_objects);
        let print = &gc.document_info().fonts;
        write_pdf_page_underlines(print, infos, &page.content, &mut contents)?;
        let mut contents = contents.start_text(TEXT_MATRIX_SCALE_X, TEXT_MATRIX_SCALE_Y);
        write_pdf_page_text(&mut contents, print, infos, page)?;
        contents.finish().map_err(Error::Contents)
    }?;
    let resources = Resources {
        fonts: fonts.into(),
        x_objects: Resource::Immediate(Box::new(x_objects)),
        proc_sets: {
            let mut sets = vec![ProcSet::PDF, ProcSet::Text];
            if has_images {
                sets.push(ProcSet::ImageB);
            }
            sets
        },
    };
    Ok(Page {
        media_box: Rectangle::from(media_box),
        resources,
        contents,
    })
}

/// Generate a sequence of PDF pages
pub fn generate_pdf_pages<GC: GenerationContext>(
    gc: &GC,
    hnd: &mut Handle,
    overrides: &Overrides,
    font_info: &Fonts,
) -> Result<(), Error> {
    let res = &mut hnd.res;
    let pages = &mut hnd.pages;

    let (fonts, infos) = font_info.font_dict(gc.fonts());
    let font_dict = res.push_font_dict(fonts);
    for page in gc.text_pages() {
        let page_info = gc.page_at(page.index as usize).unwrap();

        let page = generate_pdf_page(gc, overrides, &infos, font_dict, page, page_info, res)?;
        pages.push(page);
    }
    Ok(())
}
