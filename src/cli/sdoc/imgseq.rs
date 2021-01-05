use color_eyre::eyre;
use image::ImageFormat;
use sdo::{
    font::FontKind,
    raster::{DrawPrintErr, Page},
    sdoc::Flags,
    sdoc::Line,
    sdoc::Te,
};

use crate::cli::font::cache::FontCache;

use super::{Document, Pos};

fn draw_chars(
    doc: &Document,
    fc: &FontCache,
    data: &[Te],
    page: &mut Page,
    x: &mut u16,
    y: u16,
) -> eyre::Result<()> {
    for te in data {
        *x += te.offset;
        match doc.print_driver {
            Some(FontKind::Editor) => {
                if let Some(eset) = doc.eset(fc, te.cset) {
                    let ch = &eset.chars[te.cval as usize];
                    let x = *x; // No skew compensation (18/15)
                    let y = y * 2;
                    match page.draw_echar(x, y, ch) {
                        Ok(()) => {}
                        Err(DrawPrintErr::OutOfBounds) => {
                            eprintln!("Char out of bounds {:?}", te);
                        }
                    }
                }
            }
            Some(FontKind::Printer(pk)) => {
                if let Some(eset) = doc.pset(fc, te.cset, pk) {
                    let ch = &eset.chars[te.cval as usize];
                    let fk = FontKind::Printer(pk); // FIXME: pattern after @-binding
                    let x = fk.scale_x(*x);
                    let y = fk.scale_y(y);
                    match page.draw_printer_char(x, y, ch) {
                        Ok(()) => {}
                        Err(DrawPrintErr::OutOfBounds) => {
                            eprintln!("Char out of bounds {:?}", te);
                        }
                    }
                }
            }
            None => {
                continue;
            }
        }
    }
    Ok(())
}

fn draw_line(
    doc: &Document,
    fc: &FontCache,
    line: &Line,
    skip: u16,
    page: &mut Page,
    pos: &mut Pos,
) -> eyre::Result<()> {
    pos.y += skip + 1;

    if line.flags.contains(Flags::FLAG) {
        println!("<F: {}>", line.extra);
    }

    if line.flags.contains(Flags::ALIG) {}

    draw_chars(doc, fc, &line.data, page, &mut pos.x, pos.y)?;

    Ok(())
}

pub fn output_print(doc: &Document, fc: &FontCache) -> eyre::Result<()> {
    let out_path = &doc.opt.out;

    for page_text in &doc.tebu {
        let index = page_text.index as usize;
        let pbuf_entry = doc.pages[index].as_ref().unwrap();

        println!("{}", page_text.skip);

        if let Some(pages) = &doc.opt.page {
            if !pages.contains(&(pbuf_entry.log_pnr as usize)) {
                continue;
            }
        }

        let (mut page, mut pos) = if let Some(print_driver) = doc.print_driver {
            let width_units: u16 = pbuf_entry.margin.left + pbuf_entry.margin.right + 20;
            let height_units: u16 =
                pbuf_entry.margin.top + pbuf_entry.lines + pbuf_entry.margin.bottom;

            let width = print_driver.scale_x(width_units);
            let height = print_driver.scale_y(height_units);

            let page = Page::new(width, height);
            let pos = Pos::new(10, 0 /*page_text.skip & 0x00FF*/);
            (page, pos)
        } else {
            println!(
                "Print Driver not set, skipping page #{}",
                pbuf_entry.log_pnr
            );
            continue;
        };

        for (skip, line) in &page_text.content {
            pos.x = 10;
            draw_line(doc, fc, line, *skip, &mut page, &mut pos)?;
        }

        for site in doc.sites.iter().filter(|x| x.page == pbuf_entry.phys_pnr) {
            println!(
                "{}x{}+{},{} of {} at {},{}",
                site.sel.w, site.sel.h, site.sel.x, site.sel.y, site.img, site.pos_x, site.pos_y
            );

            if let Some(pd) = doc.print_driver {
                let px = pd.scale_x(10 + site.pos_x);
                let w = pd.scale_x(site._3);
                let py = pd.scale_y(10 + site.pos_y - site._5 / 2);
                let h = pd.scale_y(site._4 / 2);
                let image = &doc.images[site.img as usize];
                page.draw_image(px, py, w, h, image, site.sel);
            }
        }

        let image = page.to_image();
        let file_name = format!("page-{}.png", pbuf_entry.log_pnr);
        println!("Saving {}", file_name);
        let page_path = out_path.join(&file_name);
        image.save_with_format(&page_path, ImageFormat::Png)?;
    }
    Ok(())
}
