use std::path::PathBuf;

use color_eyre::eyre;
use image::ImageFormat;
use signum::{
    chsets::{
        cache::{ChsetCache, DocumentFontCacheInfo},
        printer::PrinterKind,
        FontKind,
    },
    docs::tebu::{Char, Flags, Line},
    raster::{DrawPrintErr, Page},
};

use super::{Document, Pos};

fn draw_chars(
    fc: &ChsetCache,
    print: &DocumentFontCacheInfo,
    pd: Option<FontKind>,
    data: &[Char],
    page: &mut Page,
    x: &mut u16,
    y: u16,
) {
    for te in data {
        *x += te.offset;
        match pd {
            Some(FontKind::Editor) => {
                print_echar(print, fc, te, x, y, page);
            }
            Some(FontKind::Printer(pk)) => {
                print_pchar(print, fc, te, pk, x, y, page);
            }
            None => {
                continue;
            }
        }
    }
}

fn print_pchar(
    print: &DocumentFontCacheInfo,
    fc: &ChsetCache,
    te: &Char,
    pk: PrinterKind,
    x: &mut u16,
    y: u16,
    page: &mut Page,
) {
    if let Some(eset) = print.pset(fc, te.cset, pk) {
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

fn print_echar(
    print: &DocumentFontCacheInfo,
    fc: &ChsetCache,
    te: &Char,
    x: &mut u16,
    y: u16,
    page: &mut Page,
) {
    if let Some(eset) = print.eset(fc, te.cset) {
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

fn draw_line(
    fc: &ChsetCache,
    print: &DocumentFontCacheInfo,
    pd: Option<FontKind>,
    line: &Line,
    skip: u16,
    page: &mut Page,
    pos: &mut Pos,
) {
    pos.y += skip + 1;

    if line.flags.contains(Flags::FLAG) {
        println!("<F: {}>", line.extra);
    }

    if line.flags.contains(Flags::ALIG) { /* ? */ }

    draw_chars(fc, print, pd, &line.data, page, &mut pos.x, pos.y);
}

pub fn output_print(
    doc: &Document,
    fc: &ChsetCache,
    print: &DocumentFontCacheInfo,
    pd: Option<FontKind>,
) -> eyre::Result<()> {
    let out_path: PathBuf = if let Some(path) = &doc.opt.out {
        path.clone()
    } else {
        let dir = doc.opt.file.with_extension("sdo.out");
        std::fs::create_dir(&dir)?;
        dir
    };

    for page_text in &doc.tebu {
        let index = page_text.index as usize;
        let pbuf_entry = doc.pages[index].as_ref().unwrap();

        println!("{}", page_text.skip);

        if let Some(pages) = &doc.opt.page {
            if !pages.contains(&(pbuf_entry.log_pnr as usize)) {
                continue;
            }
        }

        let (mut page, mut pos) = if let Some(print_driver) = pd {
            let width_units: u16 = pbuf_entry.format.left + pbuf_entry.format.right + 20;
            let height_units: u16 =
                pbuf_entry.format.header + pbuf_entry.format.length + pbuf_entry.format.footer;

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
            draw_line(fc, print, pd, line, *skip, &mut page, &mut pos);
        }

        for site in doc.sites.iter().filter(|x| x.page == pbuf_entry.phys_pnr) {
            println!(
                "{}x{}+{},{} of {} at {},{}",
                site.sel.w, site.sel.h, site.sel.x, site.sel.y, site.img, site.site.x, site.site.y
            );

            if let Some(pd) = pd {
                let px = pd.scale_x(10 + site.site.x);
                let w = pd.scale_x(site.site.w);
                let py = pd.scale_y(10 + site.site.y - site._5 / 2);
                let h = pd.scale_y(site.site.h / 2);
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
