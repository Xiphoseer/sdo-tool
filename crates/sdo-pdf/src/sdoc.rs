use std::io::{self, Write};

use pdf_create::{common::MediaBox, write::write_string};
use signum::{
    chsets::cache::DocumentFontCacheInfo,
    docs::{hcim::ImageSite, pbuf, tebu::PageText, Overrides},
};

use crate::font::FontInfo;

/// The `Contents` stream of a PDF
#[derive(Default)]
pub struct Contents {
    top: f32,
    left: f32,
    inner: Vec<u8>,
}

impl Contents {
    pub fn for_page(
        page_info: &pbuf::Page,
        media_box: &MediaBox,
        overrides: &Overrides,
    ) -> Contents {
        let width = page_info.format.width() * 72 / 90;
        let height = page_info.format.length as i32 * 72 / 54;

        assert!(width as i32 <= media_box.width, "Please file a bug!");

        let xmargin = (media_box.width - width as i32) / 2;
        let ymargin = (media_box.height - height) / 2;

        let left = {
            let left = xmargin as f32 + overrides.xoffset as f32;
            left - page_info.format.left as f32 * 8.0 / 10.0
        };
        let top = {
            let top = ymargin as f32 + overrides.yoffset as f32;
            media_box.height as f32 - top - 8.0
        };

        Contents::new(top, left)
    }

    /// Create a new stream
    pub fn new(top: f32, left: f32) -> Self {
        let mut inner = Vec::new();
        writeln!(inner, "0 g").unwrap();
        Self { inner, top, left }
    }

    pub fn image(&mut self, site: &ImageSite, key: &str) -> io::Result<()> {
        writeln!(self.inner, "q")?;
        let t = self.top - (((site.site.y + site.site.h / 2 - site._5 / 2) as f32 * 72.0) / 54.0);
        let l = self.left + ((site.site.x as f32 * 72.0) / 90.0);
        let w = (site.site.w as f32 * 72.0) / 90.0;
        let h = (site.site.h as f32 * /*72.0*/ 36.0) / 54.0;
        writeln!(self.inner, "{} 0 0 {} {} {} cm", w, h, l, t)?;
        writeln!(self.inner, "/{} Do", key)?;
        writeln!(self.inner, "Q")?;
        Ok(())
    }

    pub fn start_text(self, scale_x: f32, scale_y: f32) -> TextContents {
        let mut inner = self.inner;
        let left = self.left;
        let top = self.top;
        write!(
            inner,
            "q\nBT\n{} 0 0 {} {} {} Tm\n",
            scale_x, scale_y, left, top
        )
        .unwrap();
        TextContents {
            line_started: false,
            pos_y: 0,
            line_y: 0,
            line_x: 0,
            buf: vec![],
            open: false,
            needs_space: false,
            //is_ascii: true,
            cset: 0xff,
            fs: 0,
            fw: 100,
            inner,
        }
    }
}

/// Helper to create a valid `/Contents` stream
pub struct TextContents {
    buf: Vec<u8>,
    inner: Vec<u8>,
    cset: u8,
    /// The current font size
    fs: u8,
    /// The current horizontal scaling
    fw: u8,
    open: bool,
    needs_space: bool,
    //is_ascii: bool,
    line_started: bool,

    /// The vertical position in 1/216 == 1/(18*3*4) inches
    pos_y: u32,

    line_y: u32,
    line_x: u32,
}

pub fn write_pdf_page(
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

impl TextContents {
    /// Moves to the next line.
    ///
    /// `x` and `y` are in Signum coordinate units, i.e. `x` uses 1/90th of a inch and `y` uses 1/54th of an inch.
    pub fn next_line(&mut self, x: u32, y: u32) {
        self.line_x += x;
        self.line_y += y * 4;
        self.line_started = false;
    }

    fn start_line(&mut self) {
        if !self.line_started {
            self.line_started = true;
            let diff_y = (self.line_y - self.pos_y) as f32;
            writeln!(self.inner, "{} {} Td", self.line_x, diff_y / 3.0).unwrap();
            self.pos_y = self.line_y;
        }
    }

    pub fn cset(&mut self, cset: u8, font_size: u8) {
        if self.cset != cset || self.fs != font_size {
            self.cset = cset;
            self.fs = font_size;
            self.flush();
            writeln!(self.inner, "/C{} {} Tf", cset, font_size).unwrap();
        }
    }

    pub fn fwidth(&mut self, font_width: u8) {
        if self.fw != font_width {
            self.fw = font_width;
            self.flush();
            writeln!(self.inner, " {} Tz", font_width).unwrap();
        }
    }

    /// xoff in font-units (1/72000)
    pub fn xoff(&mut self, xoff: i32) {
        self.open();
        self.buf_flush();
        if self.needs_space {
            write!(self.inner, " ").unwrap();
        }
        write!(self.inner, "{}", xoff).unwrap();
        self.needs_space = true;
    }

    pub fn byte(&mut self, byte: u8) {
        self.open();
        self.buf.push(byte);
    }

    fn buf_flush(&mut self) {
        if self.buf.is_empty() {
            return;
        }
        /*if self.is_ascii {
            self.inner.push('(');
            for b in self.buf.drain(..) {
                if matches!(b, 0x28 | 0x29 | 0x5c) {
                    self.inner.push('\\');
                }
                self.inner.push(b as char);
            }
            self.inner.push(')');
        } else {
            write!(self.inner, "<").unwrap();
            for byte in self.buf.drain(..) {
                write!(self.inner, "{:02X}", byte).unwrap();
            }
            write!(self.inner, ">").unwrap();
        }*/
        write_string(&self.buf, &mut self.inner).unwrap();
        self.buf.clear();
        //self.is_ascii = true;
        self.needs_space = false;
    }

    fn open(&mut self) {
        if !self.open {
            self.start_line();
            write!(self.inner, "[").unwrap();
            self.open = true;
            self.needs_space = false;
        }
    }

    pub fn flush(&mut self) {
        if self.open {
            self.open = false;
            self.buf_flush();
            writeln!(self.inner, "] TJ").unwrap();
        }
    }

    pub fn into_inner(mut self) -> Vec<u8> {
        self.inner.extend_from_slice(b"ET\n");
        self.inner.extend_from_slice(b"Q\n");
        self.inner
    }
}
