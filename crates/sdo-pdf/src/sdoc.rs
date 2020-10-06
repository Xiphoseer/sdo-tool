use std::fmt::Write;

pub struct Contents {
    buf: Vec<u8>,
    inner: String,
    cset: u8,
    open: bool,
    needs_space: bool,
    is_ascii: bool,
    line_started: bool,
    line_y: f32,
    line_x: f32,
}

impl Contents {
    pub fn new(left: f32, top: f32) -> Self {
        Contents {
            line_started: false,
            line_y: 0.0,
            line_x: 0.0,
            buf: vec![],
            open: false,
            needs_space: false,
            is_ascii: true,
            cset: 0xff,
            inner: format!("0 g\nBT\n1 0 0 -1 {} {} Tm\n", left, top),
        }
    }

    pub fn next_line(&mut self, x: f32, y: f32) {
        self.line_x += x;
        self.line_y += y;
        self.line_started = false;
    }

    fn start_line(&mut self) {
        if !self.line_started {
            self.line_started = true;
            writeln!(self.inner, "{} {} Td", self.line_x, self.line_y).unwrap();
            self.line_y = 0.0;
        }
    }

    pub fn cset(&mut self, cset: u8) {
        if self.cset != cset {
            self.cset = cset;
            self.flush();
            writeln!(self.inner, "/C{} 2 Tf", cset).unwrap();
        }
    }

    pub fn xoff(&mut self, xoff: isize) {
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
        self.is_ascii = self.is_ascii && (byte > 31) && (byte < 127);
    }

    fn buf_flush(&mut self) {
        if self.buf.is_empty() {
            return;
        }
        if self.is_ascii {
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
        }
        self.is_ascii = true;
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

    pub fn into_inner(mut self) -> String {
        self.inner.push_str("ET\n");
        self.inner
    }
}
