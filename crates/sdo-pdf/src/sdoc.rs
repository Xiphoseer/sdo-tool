use std::io::{self, Write};

use pdf_create::write::write_string;
use signum::docs::hcim::ImageSite;

use crate::font::FontVariant;

/// The `Contents` stream of a PDF
#[derive(Default)]
pub struct Contents {
    top: f32,
    left: f32,
    inner: Vec<u8>,
}

impl Contents {
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
        let h = (site.site.h as f32 * 36.0) / 54.0;
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
            fv: FontVariant::Regular,
            fw: 100,
            leading: 0.0,
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
    /// The current font variant
    fv: FontVariant,
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

    leading: f32,
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

    fn start_line(&mut self) -> io::Result<()> {
        if !self.line_started {
            self.line_started = true;
            let diff_y = (self.line_y - self.pos_y) as f32;
            let new_leading = diff_y / 3.0;
            if new_leading == self.leading && self.line_x == 0 {
                write!(self.inner, "T*")?;
                self.needs_space = true;
            } else {
                writeln!(self.inner, "{} {} TD", self.line_x, new_leading)?;
            }
            self.leading = new_leading;
            self.pos_y = self.line_y;
        }
        Ok(())
    }

    pub fn cset(&mut self, cset: u8, font_size: u8, font_variant: FontVariant) {
        if self.cset != cset || self.fs != font_size || self.fv != font_variant {
            // Overwrite the old state
            self.cset = cset;
            self.fs = font_size;
            self.fv = font_variant;

            // Get the new font resource identifier
            let var = match font_variant {
                FontVariant::Regular => 'C',
                FontVariant::Italic => 'I',
                FontVariant::Bold => 'B',
                FontVariant::BoldItalic => 'X',
            };

            // Write to output
            self.flush();
            writeln!(self.inner, "/{}{} {} Tf", var, cset, font_size).unwrap();
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
    pub fn xoff(&mut self, xoff: i32) -> io::Result<()> {
        self.open()?;
        self.buf_flush();
        if self.needs_space {
            write!(self.inner, " ")?;
        }
        write!(self.inner, "{}", xoff)?;
        self.needs_space = true;
        Ok(())
    }

    pub fn byte(&mut self, byte: u8) -> io::Result<()> {
        self.open()?;
        self.buf.push(byte);
        Ok(())
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

    fn open(&mut self) -> io::Result<()> {
        if !self.open {
            self.start_line()?;
            write!(self.inner, "[")?;
            self.open = true;
            self.needs_space = false;
        }
        Ok(())
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
