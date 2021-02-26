use color_eyre::eyre;
use signum::{
    chsets::{cache::ChsetCache, encoding::antikro},
    docs::tebu::{Char, Flags, Line, Style},
};

use crate::cli::opt::Format;

use super::Document;

fn print_tebu_data(doc: &Document, fc: &ChsetCache, data: &[Char]) {
    let mut last_char_width: u8 = 0;
    let mut style = Style::default();

    for (_index, k) in data.iter().copied().enumerate() {
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

        let width = if let Some(eset) = &doc.eset(fc, k.cset) {
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

pub fn print_line(doc: &Document, fc: &ChsetCache, line: &Line, skip: u16) {
    if line.flags.contains(Flags::FLAG) && doc.opt.format == Format::Html {
        println!("<F: {}>", line.extra);
    }

    if line.flags.contains(Flags::PARA) && doc.opt.format == Format::Html {
        print!("<p>");
    }

    print_tebu_data(doc, fc, &line.data);

    if line.flags.contains(Flags::ALIG) && doc.opt.format == Format::Html {
        print!("<A>");
    }

    if line.flags.contains(Flags::LINE) && doc.opt.format == Format::Html {
        print!("<br>");
    }

    if doc.opt.format == Format::Plain {
        println!();
    } else {
        println!("{{{}}}", skip);
    }
}

pub fn output_console(doc: &Document, fc: &ChsetCache) -> eyre::Result<()> {
    for page_text in &doc.tebu {
        let index = page_text.index as usize;
        let pbuf_entry = doc.pages[index].as_ref().unwrap();
        println!(
            "{:04X} ----------------- [PAGE {} ({})] -------------------",
            page_text.skip, pbuf_entry.log_pnr, pbuf_entry.phys_pnr
        );
        for (skip, line) in &page_text.content {
            print_line(doc, fc, line, *skip);
        }
        println!(
            "{:04X} -------------- [END OF PAGE {} ({})] ---------------",
            page_text.skip, pbuf_entry.log_pnr, pbuf_entry.phys_pnr
        );
    }
    Ok(())
}
