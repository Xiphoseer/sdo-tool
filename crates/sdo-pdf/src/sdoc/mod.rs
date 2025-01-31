//! # Signum! Documents

use std::io;

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
    font::{encode_byte, FontInfo, Fonts, DEFAULT_FONT_SIZE, FONTUNITS_PER_SIGNUM_X},
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

        let mut prev_width: u32 = 0;

        for te in &line.data {
            let offset = te.offset as u32 * FONTUNITS_PER_SIGNUM_X;

            let is_wide = te.style.wide;
            let is_tall = te.style.tall;
            let is_small = te.style.small;

            let font_size = if is_tall {
                4
            } else if is_small {
                1
            } else {
                2
            };
            let font_width = if is_tall {
                match is_wide {
                    true => 100,
                    false => 50,
                }
            } else if is_small {
                match is_wide {
                    true => 400,
                    false => 200,
                }
            } else {
                match is_wide {
                    true => 200,
                    false => 100,
                }
            };

            let csu = te.cset as usize;
            let fi = infos[csu].ok_or_else(|| {
                let font_name = print
                    .font_cache_info_at(csu)
                    .and_then(FontCacheInfo::name)
                    .unwrap_or("");
                Error::MissingFont(csu, font_name.to_owned())
            })?;
            let width = {
                let w = fi.width(te.cval) * (DEFAULT_FONT_SIZE as u32);
                if is_wide {
                    w * 2
                } else {
                    w
                }
            };

            // FIXME: font_size is multiplied by 0.5 to support "small"
            contents.cset(te.cset, font_size).map_err(Error::Contents)?;
            contents.fwidth(font_width).map_err(Error::Contents)?;

            let mut diff = (offset as i32) - (prev_width as i32);
            if diff != 0 {
                if is_wide {
                    diff /= 2;
                }
                contents.xoff(-diff).map_err(Error::Contents)?;
            }

            // Note: slant has to be _after_ x-offset adjustment
            contents.slant(te.style.italic).map_err(Error::Contents)?;

            let win_ansi_byte = encode_byte(te.cval);
            contents
                .byte(win_ansi_byte, width)
                .map_err(Error::Contents)?;

            prev_width = width;
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

        let image = image_for_site(gc.document_info(), site);

        contents.image(site, &key).unwrap();
        x_objects.insert(key.clone(), res.push_xobject(image).into());
        has_images |= true;
    }
    has_images
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
    let media_box = MediaBox::A4;
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

        let page = generate_pdf_page(
            gc,
            overrides,
            &infos,
            font_dict.clone(),
            page,
            page_info,
            res,
        )?;
        pages.push(page);
    }
    Ok(())
}
