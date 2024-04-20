use crate::{
    chsets::{
        cache::{ChsetCache, DocumentFontCacheInfo},
        printer::PrinterKind,
        FontKind,
    },
    docs::{hcim, pbuf, tebu},
    util::Pos,
};

use super::{DrawPrintErr, Page};

fn print_pchar(
    print: &DocumentFontCacheInfo,
    fc: &ChsetCache,
    te: &tebu::Char,
    pk: PrinterKind,
    x: &mut u16,
    y: u16,
    page: &mut Page,
) {
    if let Some(pset) = print.pset(fc, te.cset, pk) {
        let x = pk.scale_x(*x);
        let y = pk.scale_y(y);
        if let Err(DrawPrintErr::OutOfBounds) =
            page.draw_printer_char(x, y, &pset.chars[te.cval as usize])
        {
            log::error!("Char out of bounds {:?}", te);
        }
    }
}

fn print_echar(
    print: &DocumentFontCacheInfo,
    fc: &ChsetCache,
    te: &tebu::Char,
    x: &mut u16,
    y: u16,
    page: &mut Page,
) {
    if let Some(eset) = print.eset(fc, te.cset) {
        let x = *x; // No skew compensation (18/15)
        let y = y * 2;
        if let Err(DrawPrintErr::OutOfBounds) = page.draw_echar(x, y, &eset.chars[te.cval as usize])
        {
            log::error!("Char out of bounds {:?}", te);
        }
    }
}

/// Print a single page
pub fn render_doc_page(
    page_text: &tebu::PageText,
    pbuf_entry: &pbuf::Page,
    image_sites: &[hcim::ImageSite],
    images: &[(String, Page)],
    pd: FontKind,
    fc: &ChsetCache,
    print: &DocumentFontCacheInfo,
) -> Page {
    let fmt = &pbuf_entry.format;
    let width = pd.scale_x(fmt.left + fmt.right + 20);
    let height = pd.scale_y(fmt.header + fmt.length + fmt.footer);

    let mut page = Page::new(width, height);
    let mut pos = Pos::new(10, 0);

    #[allow(clippy::type_complexity)]
    let print_char: Box<dyn Fn(&tebu::Char, &mut u16, u16, &mut Page)> = match pd {
        FontKind::Editor => Box::new(move |te, x, y, p| print_echar(print, fc, te, x, y, p)),
        FontKind::Printer(pk) => {
            Box::new(move |te, x, y, p| print_pchar(print, fc, te, pk, x, y, p))
        }
    };
    for (skip, line) in &page_text.content {
        pos.x = 10;
        pos.y += skip + 1;
        for te in &line.data {
            pos.x += te.offset;
            (print_char)(te, &mut pos.x, pos.y, &mut page);
        }
    }
    for site in image_sites.iter().filter(|x| x.page == pbuf_entry.phys_pnr) {
        log::debug!(
            "{}x{}+{},{} of {} at {},{}",
            site.sel.w,
            site.sel.h,
            site.sel.x,
            site.sel.y,
            site.img,
            site.site.x,
            site.site.y
        );

        let px = pd.scale_x(10 + site.site.x);
        let w = pd.scale_x(site.site.w);
        let py = pd.scale_y(10 + site.site.y - site._5 / 2);
        let h = pd.scale_y(site.site.h / 2);
        let (_, image) = &images[site.img as usize];
        page.draw_image(px, py, w, h, image, site.sel);
    }
    page
}
