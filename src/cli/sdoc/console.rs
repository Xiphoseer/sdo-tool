use color_eyre::eyre;
use prettytable::{cell, format, row, Cell, Row, Table};
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

use crate::cli::opt::Format;

use super::Document;

fn print_tebu_data(print: &DocumentFontCacheInfo, fc: &ChsetCache, data: &[Char]) {
    let mut last_char_width: u8 = 0;
    let mut style = Style::default();

    for k in data {
        let chr = antikro::decode(k.cval);
        if chr == '\0' {
            println!("<NUL:{}>", k.offset);
            continue;
        }

        if !k.style.bold && style.bold {
            style.bold = false;
            print!("</b>");
        }
        if !k.style.italic && style.italic {
            style.italic = false;
            print!("</i>");
        }
        if !k.style.tall && style.tall {
            style.tall = false;
            print!("</sth2>");
        }
        if !k.style.wide && style.wide {
            style.wide = false;
            print!("</sth1>");
        }
        if !k.style.small && style.small {
            style.small = false;
            print!("</small>");
        }

        let lcw = last_char_width.into();
        if k.offset >= lcw {
            let mut space = k.offset - lcw;

            while space >= 7 {
                print!(" ");
                space -= 7;
            }
        }

        if k.style.footnote {
            print!("<footnote>");
        }
        if k.style.small && !style.small {
            style.small = true;
            print!("<small>");
        }
        if k.style.wide && !style.wide {
            style.wide = true;
            print!("<sth1>");
        }
        if k.style.tall && !style.tall {
            style.tall = true;
            print!("<sth2>");
        }
        if k.style.italic && !style.italic {
            style.italic = true;
            print!("<i>");
        }
        if k.style.bold && !style.bold {
            style.bold = true;
            print!("<b>");
        }

        let width = if let Some(eset) = print.eset(fc, k.cset) {
            eset.chars[k.cval as usize].width
        } else {
            // default for fonts that are missing
            antikro::WIDTH[k.cval as usize]
        };
        last_char_width = if chr == '\n' { 0 } else { width };
        if (0xE000..=0xE080).contains(&(chr as u32)) {
            print!("<C{}>", (chr as u32) - 0xE000);
        } else if (0x1FBF0..=0x1FBF9).contains(&(chr as u32)) {
            print!("[{}]", chr as u32 - 0x1FBF0);
        } else {
            if k.style.underlined {
                print!("\u{0332}");
            }
            print!("{}", chr);
        }
    }
    if style.bold {
        print!("</b>");
    }
    if style.italic {
        print!("</i>");
    }
    if style.tall {
        print!("</sth2>");
    }
    if style.wide {
        print!("</sth1>");
    }
    if style.small {
        print!("</small>");
    }
}

pub fn print_line(
    format: Format,
    print: &DocumentFontCacheInfo,
    fc: &ChsetCache,
    line: &Line,
    skip: u16,
) {
    if line.flags.contains(Flags::FLAG) && format == Format::Html {
        println!("<F: {}>", line.extra);
    }

    if line.flags.contains(Flags::PARA) && format == Format::Html {
        print!("<p>");
    }

    print_tebu_data(print, fc, &line.data);

    if line.flags.contains(Flags::ALIG) && format == Format::Html {
        print!("<A>");
    }

    if line.flags.contains(Flags::LINE) && format == Format::Html {
        print!("<br>");
    }

    if format == Format::Plain {
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
    fc: &ChsetCache,
    print: &DocumentFontCacheInfo,
) -> eyre::Result<()> {
    print_pages(&doc.pages[..]);

    for page_text in &doc.tebu.pages {
        let index = page_text.index as usize;
        let pbuf_entry = doc.pages[index].as_ref().unwrap();
        println!(
            "{:04X} ----------------- [PAGE {} ({})] -------------------",
            page_text.skip, pbuf_entry.log_pnr, pbuf_entry.phys_pnr
        );
        for (skip, line) in &page_text.content {
            print_line(doc.opt.format, print, fc, line, *skip);
        }
        println!(
            "{:04X} -------------- [END OF PAGE {} ({})] ---------------",
            page_text.skip, pbuf_entry.log_pnr, pbuf_entry.phys_pnr
        );
    }

    if !doc.sites.is_empty() {
        print_img_sites(&doc.sites[..]);
    }

    Ok(())
}
