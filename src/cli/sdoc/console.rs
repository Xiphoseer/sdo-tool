use color_eyre::eyre;
use prettytable::{format, row, Cell, Row, Table};
use signum::{
    chsets::{
        cache::{ChsetCache, DocumentFontCacheInfo},
        encoding::antikro,
    },
    docs::{
        hcim::ImageSite,
        pbuf::Page,
        tebu::{Char, Flags, Line, Style},
    },
};

use crate::cli::opt::{Format, Options};

use super::Document;

fn print_tebu_data(
    print: &DocumentFontCacheInfo,
    fc: &ChsetCache,
    data: &[Char],
    space_width: u16,
) {
    let mut last_char_width: u8 = 0;
    let mut style = Style::default();

    for k in data {
        let chr = antikro::decode(k.cval);
        if chr == '\0' {
            println!("<NUL:{}>", k.offset);
            continue;
        }

        if !k.style.is_bold() && style.is_bold() {
            style.remove(Style::BOLD);
            print!("</b>");
        }
        if !k.style.is_italic() && style.is_italic() {
            style.remove(Style::ITALIC);
            print!("</i>");
        }
        if !k.style.is_tall() && style.is_tall() {
            style.remove(Style::TALL);
            print!("</tall>");
        }
        if !k.style.is_wide() && style.is_wide() {
            style.remove(Style::WIDE);
            print!("</wide>");
        }
        if !k.style.is_small() && style.is_small() {
            style.remove(Style::SMALL);
            print!("</small>");
        }

        let lcw = last_char_width.into();
        if k.offset >= lcw {
            let mut space = k.offset - lcw;

            while space > 2 {
                print!(" ");
                if space >= space_width {
                    space -= space_width;
                } else {
                    space = 0;
                }
            }
        }

        if k.style.is_footnote() {
            print!("<footnote>");
        }
        if k.style.is_small() && !style.is_small() {
            style.insert(Style::SMALL);
            print!("<small>");
        }
        if k.style.is_wide() && !style.is_wide() {
            style.insert(Style::WIDE);
            print!("<wide>");
        }
        if k.style.is_tall() && !style.is_tall() {
            style.insert(Style::TALL);
            print!("<tall>");
        }
        if k.style.is_italic() && !style.is_italic() {
            style.insert(Style::ITALIC);
            print!("<i>");
        }
        if k.style.is_bold() && !style.is_bold() {
            style.insert(Style::BOLD);
            print!("<b>");
        }

        let width = if let Some(eset) = print.eset(fc, k.cset) {
            eset.chars[k.cval as usize].width
        } else {
            // default for fonts that are missing
            antikro::WIDTH[k.cval as usize]
        };
        last_char_width = if chr == '\n' { 0 } else { width };
        if k.style.is_wide() {
            last_char_width *= 2;
        }
        if (0xE000..=0xE080).contains(&(chr as u32)) {
            print!("<C{}>", (chr as u32) - 0xE000);
        } else if (0x1FBF0..=0x1FBF9).contains(&(chr as u32)) {
            print!("[{}]", chr as u32 - 0x1FBF0);
        } else {
            if k.style.is_underlined() {
                print!("\u{0332}");
            }
            print!("{}", chr);
        }
    }
    if style.is_bold() {
        print!("</b>");
    }
    if style.is_italic() {
        print!("</i>");
    }
    if style.is_tall() {
        print!("</tall>");
    }
    if style.is_wide() {
        print!("</wide>");
    }
    if style.is_small() {
        print!("</small>");
    }
}

pub fn print_line(
    is_html: bool,
    is_plain: bool,
    print: &DocumentFontCacheInfo,
    fc: &ChsetCache,
    line: &Line,
    skip: u16,
    space_width: u16,
) {
    if line.flags.contains(Flags::FLAG) && is_html {
        println!("<F: {}>", line.extra);
    }

    if line.flags.contains(Flags::PARA) && is_html {
        print!("<p>");
    }

    print_tebu_data(print, fc, &line.data, space_width);

    if line.flags.contains(Flags::ALIG) && is_html {
        print!("<A>");
    }

    if line.flags.contains(Flags::LINE) && is_html {
        print!("<br>");
    }

    if is_plain {
        println!();
    } else {
        println!("{{{}}}", skip);
    }
}

pub fn print_pages(pages: &[Option<Page>]) {
    // Create the table
    let mut page_table = Table::new();
    page_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    // Add a row per time
    page_table.set_titles(row![
        "idx", "#phys", "#log", "len", "left", "right", "head", "foot", "numbpos", "kapitel",
        "???", "#vi" //, "rest",
    ]);

    for (index, pbuf_entry) in pages.iter().enumerate() {
        if let Some(page) = pbuf_entry {
            page_table.add_row(row![
                index,
                page.phys_pnr,
                page.log_pnr,
                page.format.length,
                page.format.left,
                page.format.right,
                page.format.header,
                page.format.footer,
                page.numbpos,
                page.kapitel,
                page.intern,
                page.vis_pnr,
                //buf,
            ]);
        } else {
            page_table.add_row(row![
                index, "---", "---", "---", "---", "---", "---", "---", "---", "---", "---",
                "---" //, "---"
            ]);
        }
    }

    // Print the table to stdout
    page_table.printstd();
}

fn print_img_sites(sites: &[ImageSite]) {
    let mut image_table = Table::new();
    image_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    // Add a row per time
    image_table.set_titles(row![
        "page", "pos_x", "pos_y", "site_w", "site_h", "[5]", "sel_x", "sel_y", "sel_w", "sel_h",
        "[A]", "[B]", "[C]", "img", "[E]", "[F]",
    ]);

    for isite in sites {
        image_table.add_row(Row::new(vec![
            Cell::new(&format!("{}", isite.page)),
            Cell::new(&format!("{}", isite.site.x)),
            Cell::new(&format!("{}", isite.site.y)),
            Cell::new(&format!("{}", isite.site.w)),
            Cell::new(&format!("{}", isite.site.h)),
            Cell::new(&format!("{}", isite._5)),
            Cell::new(&format!("{}", isite.sel.x)),
            Cell::new(&format!("{}", isite.sel.y)),
            Cell::new(&format!("{}", isite.sel.w)),
            Cell::new(&format!("{}", isite.sel.h)),
            Cell::new(&format!("{}", isite._A)),
            Cell::new(&format!("{}", isite._B)),
            Cell::new(&format!("{}", isite._C)),
            Cell::new(&format!("{}", isite.img)),
            Cell::new(&format!("{}", isite._E)),
            Cell::new(&format!("{:?}", isite._F)),
        ]));
    }

    image_table.printstd();
}

pub fn output_console(
    doc: &Document,
    opt: &Options,
    fc: &ChsetCache,
    print: &DocumentFontCacheInfo,
) -> eyre::Result<()> {
    print_pages(&doc.pages[..]);

    let is_html = opt.format == Format::Html;
    let is_plain = opt.format == Format::Plain;

    let space_width = doc.sysp.as_ref().map(|sysp| sysp.space_width).unwrap_or(7);

    for page_text in &doc.tebu.pages {
        let index = page_text.index as usize;
        let pbuf_entry = doc.pages[index].as_ref().unwrap();
        println!(
            "{:04X} ----------------- [PAGE {} ({})] -------------------",
            page_text.skip, pbuf_entry.log_pnr, pbuf_entry.phys_pnr
        );
        for (skip, line) in &page_text.content {
            print_line(is_html, is_plain, print, fc, line, *skip, space_width);
        }
        println!(
            "{:04X} -------------- [END OF PAGE {} ({})] ---------------",
            page_text.skip, pbuf_entry.log_pnr, pbuf_entry.phys_pnr
        );
    }

    if let Some(hcim) = &doc.hcim {
        if !hcim.sites.is_empty() {
            print_img_sites(&hcim.sites[..]);
        }
    }

    Ok(())
}
