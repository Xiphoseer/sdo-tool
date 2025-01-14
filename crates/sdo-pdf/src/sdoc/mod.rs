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
    font::{FontInfo, Fonts, FONTUNITS_PER_SIGNUM_X},
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
                let w = fi.width(te.cval);
                if is_wide { w * 2 } else { w }
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
            contents.byte(te.cval, width).map_err(Error::Contents)?;

            prev_width = width;
        }

        contents.flush().map_err(Error::Contents)?;
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
        let mut contents = contents.start_text(TEXT_MATRIX_SCALE_X, TEXT_MATRIX_SCALE_Y);
        write_pdf_page_text(&mut contents, &gc.document_info().fonts, infos, page)?;
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
