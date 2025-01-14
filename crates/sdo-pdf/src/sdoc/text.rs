use std::io;

use pdf_create::write::write_string;

/// Constant to get from 1/216th inches (y space) to 1/72th space (PDF space)
const Y_SCALE_INVERSE: f32 = 3.0;

pub(crate) const TEXT_MATRIX_SCALE_X: f32 = 1.0;
pub(crate) const TEXT_MATRIX_SCALE_Y: f32 = 1.0;

/// Helper to create a valid `/Contents` stream
pub struct TextContents<O> {
    buf: Vec<u8>,
    /// The output that we write to
    inner: O,
    cset: u8,
    /// The current font size
    fs: u8,
    /// The current horizontal scaling
    fw: u16,

    /// slant
    slant: f32,

    open: bool,
    needs_space: bool,
    //is_ascii: bool,
    line_started: bool,

    /// The horizontal position in 1/72000 inches of the *text line matrix*
    ///
    /// This is the position a new-line is relative to, so we need to reset
    /// it to 0 when we had an inline `Tm` call.
    pos_x: i32,

    /// The vertical position in 1/216 == 1/(18*3*4) inches
    pos_y: u32,

    line_y: u32,

    /// Position within the line, in fontunits (1/72000 inches)
    line_x: i32,

    // origin (top-left)
    origin: (f32, f32),

    // scale (x,y)
    scale: (f32, f32),
}

impl<O: io::Write> TextContents<O> {
    pub(super) fn new(inner: O, origin: (f32, f32), scale: (f32, f32)) -> Self {
        Self {
            line_started: false,
            pos_x: 0,
            pos_y: 0,
            line_y: 0,
            line_x: 0,
            slant: 0.0,
            buf: vec![],
            open: false,
            needs_space: false,
            cset: 0xff,
            fs: 0,
            fw: 100,
            inner,
            origin,
            scale,
        }
    }

    /// Moves to the next line.
    ///
    /// `x` and `y` are in Signum coordinate units, i.e. `x` uses 1/90th of a inch and `y` uses 1/54th of an inch.
    pub fn next_line(&mut self, x: u32, y: u32) {
        self.line_x = x as i32 * 1000;
        self.line_y += y * 4;
        self.line_started = false;
    }

    /// Start a new line (`Td` operator)
    ///
    /// `TD` would work as well, just sets the *leading* (distance between baselines) as well via implicit `-Ty TL`
    fn start_line(&mut self) -> io::Result<()> {
        if !self.line_started {
            self.line_started = true;
            let diff_y = (self.line_y - self.pos_y) as f32;
            if self.pos_x > 0 {
                // If we messed with the text line matrix, do this the long way around
                let left = self.origin.0; // FIXME
                let top = self.origin.1 - self.line_y as f32 / Y_SCALE_INVERSE;
                self.set_text_matrix(self.scale.0, 0.0, 0.0, self.scale.1, left, top)?;
            } else {
                writeln!(
                    self.inner,
                    "{} {} Td",
                    self.line_x,
                    -diff_y / Y_SCALE_INVERSE
                )?;
            }
            self.pos_y = self.line_y;
        }
        Ok(())
    }

    /// Set the font and size (`Tf` operator)
    ///
    /// Font size is in (2 x natural font size)
    pub fn cset(&mut self, cset: u8, font_size: u8) -> io::Result<()> {
        if self.cset != cset || self.fs != font_size {
            self.cset = cset;
            self.fs = font_size;
            self.flush()?;
            writeln!(self.inner, "/C{} {} Tf", cset, (font_size as f32) * 0.5)?;
        }
        Ok(())
    }

    /// Set the horizontal scaling, `Th`, to `(scale ÷ 100)`.
    ///
    /// `scale` is a number specifying the percentage of the normal width.
    ///
    /// Initial value: 100 (normal width).
    pub fn fwidth(&mut self, scale: u16) -> io::Result<()> {
        if self.fw != scale {
            self.fw = scale;
            self.flush()?;
            writeln!(self.inner, " {} Tz", scale)?;
        }
        Ok(())
    }

    /// xoff in font-units (1/72000)
    ///
    /// This amount is *subtracted* from the horizontal position
    pub fn xoff(&mut self, xoff: i32) -> io::Result<()> {
        self.open()?;
        self.buf_flush()?;
        if self.needs_space {
            write!(self.inner, " ")?;
        }
        write!(self.inner, "{}", xoff)?;
        self.line_x -= xoff;
        self.needs_space = true;
        Ok(())
    }

    /// Push a new byte, width in fontunits (1/72000 inches)
    pub fn byte(&mut self, byte: u8, width: u32) -> io::Result<()> {
        self.open()?;
        self.buf.push(byte);
        self.line_x += width as i32;
        Ok(())
    }

    fn buf_flush(&mut self) -> io::Result<()> {
        if self.buf.is_empty() {
            return Ok(());
        }
        write_string(&self.buf, &mut self.inner)?;
        self.buf.clear();
        self.needs_space = false;
        Ok(())
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

    pub fn flush(&mut self) -> io::Result<()> {
        if self.open {
            self.open = false;
            self.buf_flush()?;
            writeln!(self.inner, "] TJ")?;
        }
        Ok(())
    }

    pub fn finish(mut self) -> io::Result<O> {
        writeln!(self.inner, "ET")?;
        writeln!(self.inner, "Q")?;
        Ok(self.inner)
    }

    pub(crate) fn slant(&mut self, is_italic: bool) -> io::Result<()> {
        let slant = match is_italic {
            true => 0.25, // 1:4 slant
            false => 0.0,  // no slant
        };
        if slant != self.slant {
            self.flush()?;
            self.slant = slant;
            self.pos_x = self.line_x;
            let left = self.origin.0 + self.line_x as f32 / 1000.0; // FIXME
            let top = self.origin.1 - self.pos_y as f32 / Y_SCALE_INVERSE;
            self.set_text_matrix(self.scale.0, 0.0, self.slant, self.scale.1, left, top)?;
        }
        Ok(())
    }

    /// Set the text matrix (`Tm` operator)
    ///
    /// ```text
    /// ⎡ a b 0 ⎤
    /// ⎢ c d 0 ⎥
    /// ⎣ e f 1 ⎦
    /// ```
    pub(crate) fn set_text_matrix(
        &mut self,
        a: f32,
        b: f32,
        c: f32,
        d: f32,
        e: f32,
        f: f32,
    ) -> io::Result<()> {
        writeln!(self.inner, "{a} {b} {c} {d} {e} {f} Tm")?;
        Ok(())
    }

    pub(crate) fn goto_origin(&mut self) -> io::Result<()> {
        self.set_text_matrix(
            self.scale.0,
            0.0,
            0.0,
            self.scale.1,
            self.origin.0,
            self.origin.1,
        )?;
        Ok(())
    }
}
