use std::io::Write;

use pdf_create::write::write_string;

/// Helper to create a valid `/Contents` stream
pub struct Contents {
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

impl Contents {
    pub fn new(scale_x: f32, scale_y: f32, left: f32, top: f32) -> Self {
        let inner = format!("0 g\nBT\n{} 0 0 {} {} {} Tm\n", scale_x, scale_y, left, top);
        let inner = inner.into_bytes();
        Contents {
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
        self.inner
    }
}
