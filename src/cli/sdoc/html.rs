use std::fmt::{self, Write};

use color_eyre::eyre;
use signum::{
    chsets::{cache::ChsetCache, encoding::antikro},
    docs::tebu::{Char, Flags, Line, Style},
};

use super::Document;

struct HtmlGen<'a> {
    out: String,
    doc: &'a Document<'a>,
    fc: &'a ChsetCache,

    par: bool,
    protected: bool,
    skip: u16,
}

impl<'a> HtmlGen<'a> {
    fn new(doc: &'a Document, fc: &'a ChsetCache) -> Result<Self, fmt::Error> {
        let file_name = doc.opt.file.file_name().unwrap().to_string_lossy();
        let mut out = String::new();
        writeln!(out, "<!DOCTYPE html>")?;
        writeln!(out, "<html>")?;
        writeln!(out, "  <head>")?;
        writeln!(out, "    <title>{}</title>", file_name)?;
        writeln!(out, "    <meta name=\"generator\"")?;
        writeln!(
            out,
            "          content=\"https://xiphoseer.github.io/sdo-tool\">"
        )?;
        writeln!(out, "    <style>")?;
        writeln!(out, "      .wide .tall {{")?;
        writeln!(out, "        font-size: 200%;")?;
        writeln!(out, "      }}")?;
        writeln!(out, "      .page {{")?;
        writeln!(out, "        border-bottom: 2px dotted black;")?;
        writeln!(out, "      }}")?;
        writeln!(out, "      figure {{")?;
        writeln!(out, "        border: 1px solid black;")?;
        writeln!(out, "        margin: 5px;")?;
        writeln!(out, "        padding: 5px;")?;
        writeln!(out, "        white-space: pre;")?;
        writeln!(out, "        line-height: 2px;")?;
        writeln!(out, "        font-family: monospace;")?;
        writeln!(out, "      }}")?;
        writeln!(out, "    </style>")?;
        writeln!(out, "  </head>")?;
        writeln!(out, "  <body>")?;
        Ok(Self {
            doc,
            fc,
            out,
            par: false,
            skip: 0,
            protected: false,
        })
    }

    fn finish(mut self) -> Result<String, fmt::Error> {
        writeln!(self.out, "  </body>")?;
        writeln!(self.out, "</html>")?;
        Ok(self.out)
    }

    fn print_tebu_data(&mut self, data: &[Char]) -> fmt::Result {
        let mut last_char_width: u8 = 0;
        let mut style = Style::default();

        for k in data {
            let cset = self.fc.cset(k.cset as usize);
            let mapping = cset.and_then(|c| c.map()).unwrap_or_default();
            let chr = mapping.decode(k.cval);

            if chr == '\0' {
                writeln!(self.out, "<!-- NUL -->")?;
                continue;
            }

            if !k.style.underlined && style.underlined {
                style.underlined = false;
                write!(self.out, "</u>")?;
            }
            if !k.style.bold && style.bold {
                style.bold = false;
                write!(self.out, "</b>")?;
            }
            if !k.style.italic && style.italic {
                style.italic = false;
                write!(self.out, "</i>")?;
            }
            if !k.style.tall && style.tall {
                style.tall = false;
                write!(self.out, "</span>")?;
            }
            if !k.style.wide && style.wide {
                style.wide = false;
                write!(self.out, "</span>")?;
            }
            if !k.style.small && style.small {
                style.small = false;
                write!(self.out, "</small>")?;
            }

            let lcw = last_char_width.into();
            if k.offset >= lcw {
                let mut space = k.offset - lcw;

                while space > 2 {
                    write!(self.out, " ")?;
                    if space >= 7 {
                        space -= 7;
                    } else {
                        space = 0;
                    }
                }
            }

            if k.style.footnote {
                write!(self.out, "<footnote>")?;
            }
            if k.style.small && !style.small {
                style.small = true;
                write!(self.out, "<small>")?;
            }
            if k.style.wide && !style.wide {
                style.wide = true;
                write!(self.out, "<span class=\"wide\">")?;
            }
            if k.style.tall && !style.tall {
                style.tall = true;
                write!(self.out, "<span class=\"tall\">")?;
            }
            if k.style.italic && !style.italic {
                style.italic = true;
                write!(self.out, "<i>")?;
            }
            if k.style.bold && !style.bold {
                style.bold = true;
                write!(self.out, "<b>")?;
            }
            if k.style.underlined && !style.underlined {
                style.underlined = true;
                write!(self.out, "<u>")?;
            }

            let mut width = if let Some(eset) = &self.doc.print.eset(self.fc, k.cset) {
                eset.chars[k.cval as usize].width
            } else {
                // default for fonts that are missing
                antikro::WIDTH[k.cval as usize]
            };
            if style.wide {
                width *= 2;
            }
            last_char_width = if chr == '\n' { 0 } else { width };
            if (0xE000..=0xE080).contains(&(chr as u32)) {
                write!(self.out, "<!-- C{} -->", (chr as u32) - 0xE000)?;
            } else if (0x1FBF0..=0x1FBF9).contains(&(chr as u32)) {
                write!(self.out, "{}", chr as u32 - 0x1FBF0)?;
            } else {
                write!(self.out, "{}", chr)?;
            }
        }
        if style.underlined {
            write!(self.out, "</u>")?;
        }
        if style.bold {
            write!(self.out, "</b>")?;
        }
        if style.italic {
            write!(self.out, "</i>")?;
        }
        if style.tall {
            write!(self.out, "</span>")?;
        }
        if style.wide {
            write!(self.out, "</span>")?;
        }
        if style.small {
            write!(self.out, "</small>")?;
        }
        Ok(())
    }

    pub fn print_line(&mut self, line: &Line, skip: u16) -> fmt::Result {
        if self.protected {
            for _ in 0..skip {
                writeln!(self.out)?;
            }
        } else {
            // Normal line skip
            self.skip += skip + 1;
            while self.skip > 10 {
                self.skip -= 11;
                write!(self.out, "<br>")?;
            }
        }

        if line.flags.contains(Flags::FLAG) {
            write!(self.out, "<!-- F: {} -->", line.extra)?;
        }

        if !line.flags.contains(Flags::ALIG) && self.protected {
            self.protected = false;
            write!(self.out, "</figure>")?;
        }

        if line.flags.contains(Flags::PARA) {
            if self.par {
                writeln!(self.out, "</p>")?;
            }
            self.par = true;
            write!(self.out, "<p>")?;
        }

        if line.flags.contains(Flags::LINE) {
            // FIXME: print only main lines
        }

        if line.flags.contains(Flags::ALIG) && !self.protected {
            if self.par {
                self.par = false;
                writeln!(self.out, "</p>")?;
            }
            self.protected = true;
            self.skip = 0;
            write!(self.out, "<figure>")?;
        }
        self.print_tebu_data(&line.data)?;

        writeln!(self.out)?;
        Ok(())
    }

    fn body(&mut self) -> fmt::Result {
        for page_text in &self.doc.tebu {
            let index = page_text.index as usize;
            let pbuf_entry = self.doc.pages[index].as_ref().unwrap();
            writeln!(
                self.out,
                "    <section class=\"page\" id=\"p{}\">",
                pbuf_entry.log_pnr,
            )?;
            //page_text.skip
            //pbuf_entry.phys_pnr
            for (skip, line) in &page_text.content {
                self.print_line(line, *skip)?;
            }
            if self.par {
                self.par = false;
                writeln!(self.out, "      </p>")?;
            }
            writeln!(self.out, "    </section>")?;
        }
        Ok(())
    }
}

pub fn output_html(doc: &Document, fc: &ChsetCache) -> eyre::Result<()> {
    let mut gen = HtmlGen::new(doc, fc)?;
    gen.body()?;

    let path = if let Some(out) = &doc.opt.out {
        out.clone()
    } else {
        doc.opt.file.with_extension("html")
    };

    let contents = gen.finish()?;
    std::fs::write(&path, contents)?;
    eprintln!("Wrote HTML file to '{}'", path.display());

    Ok(())
}
