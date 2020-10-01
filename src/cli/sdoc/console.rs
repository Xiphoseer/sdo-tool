use color_eyre::eyre;
use sdo::{
    font::antikro,
    sdoc::{Flags, Line, Style, Te},
};

use crate::cli::opt::Format;

use super::Document;

fn print_tebu_data(doc: &Document, data: &[Te]) {
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
        if !k.style.sth2 && style.sth2 {
            style.sth2 = false;
            print!("</sth2>");
        }
        if !k.style.sth1 && style.sth1 {
            style.sth1 = false;
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
        if k.style.sth1 && !style.sth1 {
            style.sth1 = true;
            print!("<sth1>");
        }
        if k.style.sth2 && !style.sth2 {
            style.sth2 = true;
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

        let width = if let Some(eset) = &doc.chsets_e24[k.cset as usize] {
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
    if style.sth2 {
        print!("</sth2>");
    }
    if style.sth1 {
        print!("</sth1>");
    }
    if style.small {
        print!("</small>");
    }
}

pub fn print_line(doc: &Document, line: &Line, skip: u16) {
    if line.flags.contains(Flags::FLAG) && doc.opt.format == Format::Html {
        println!("<F: {}>", line.extra);
    }

    if line.flags.contains(Flags::PARA) && doc.opt.format == Format::Html {
        print!("<p>");
    }

    print_tebu_data(doc, &line.data);

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

pub fn output_console(doc: &Document) -> eyre::Result<()> {
    for page_text in &doc.tebu {
        let index = page_text.index as usize;
        let pbuf_entry = doc.pages[index].as_ref().unwrap();
        println!(
            "{:04X} ----------------- [PAGE {} ({})] -------------------",
            page_text.skip, pbuf_entry.log_pnr, pbuf_entry.phys_pnr
        );
        for (skip, line) in &page_text.content {
            print_line(doc, line, *skip);
        }
        println!(
            "{:04X} -------------- [END OF PAGE {} ({})] ---------------",
            page_text.skip, pbuf_entry.log_pnr, pbuf_entry.phys_pnr
        );
    }
    Ok(())
}
