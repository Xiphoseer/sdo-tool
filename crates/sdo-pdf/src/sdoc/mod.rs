use pdf_create::{
    common::{MediaBox, ProcSet, Rectangle},
    high::{DictResource, Font, GlobalResource, Handle, Page, Res, Resource, Resources, XObject},
};
use signum::{
    chsets::cache::DocumentFontCacheInfo,
    docs::{
        pbuf,
        tebu::{self, PageText},
        GenerationContext, Overrides,
    },
};

mod contents;
mod text;
use contents::Contents;
use text::TextContents;

use crate::{
    font::{FontInfo, Fonts},
    image::image_for_site,
};

/// Write the text for a PDF page
fn write_pdf_page_text(
    contents: &mut TextContents,
    print: &DocumentFontCacheInfo,
    infos: &[Option<&FontInfo>; 8],
    page: &PageText,
) -> Result<(), crate::Error> {
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
                crate::Error::MissingFont(csu, font_name.to_owned())
            })?;
            prev_width = fi.width(te.cval) as i32;
            if is_wide {
                prev_width *= 2;
            }
        }

        contents.flush();
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

pub fn generate_pdf_page<GC: GenerationContext>(
    gc: &GC,
    overrides: &Overrides,
    infos: &[Option<&FontInfo>; 8],
    fonts: GlobalResource<DictResource<Font<'static>>>,
    page: &tebu::PageText,
    page_info: &pbuf::Page,
    res: &mut Res<'_>,
) -> Result<Page<'static>, crate::Error> {
    let media_box = MediaBox::A4;
    let mut x_objects = DictResource::<XObject>::new();

    let has_images: bool;
    let contents = {
        let mut contents = Contents::for_page(page_info, &media_box, overrides);
        has_images = write_pdf_page_images(&mut contents, gc, page_info, res, &mut x_objects);
        let mut contents = contents.start_text(1.0, -1.0);
        write_pdf_page_text(&mut contents, &gc.document_info().fonts, infos, page)?;
        contents.into_inner()
    };
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

pub fn generate_pdf_pages<GC: GenerationContext>(
    gc: &GC,
    hnd: &mut Handle,
    overrides: &Overrides,
    font_info: &Fonts,
) -> Result<(), crate::Error> {
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
