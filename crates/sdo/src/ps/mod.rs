use std::io::{self, Stdout, Write};

pub struct PSWriter<W: Write> {
    // chars in the current line
    lc: usize,
    // need space
    ns: bool,
    // writer
    inner: W,
}

impl PSWriter<Stdout> {
    pub fn new() -> Self {
        Self::from(std::io::stdout())
    }
}

impl Default for PSWriter<Stdout> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W: Write> From<W> for PSWriter<W> {
    fn from(inner: W) -> Self {
        Self {
            lc: 0,
            ns: false,
            inner,
        }
    }
}

impl<W: Write> PSWriter<W> {
    fn need_len(&mut self, len: usize) -> io::Result<()> {
        if self.lc + len > 72 {
            writeln!(self.inner)?;
            self.lc = 0;
        } else if self.ns {
            write!(self.inner, " ")?;
            self.lc += 1;
            self.ns = false;
        }
        Ok(())
    }

    pub fn crlf(&mut self) -> io::Result<()> {
        if self.lc > 0 {
            writeln!(self.inner)?;
            self.lc = 0;
            self.ns = false;
        }
        Ok(())
    }

    pub fn write_magic(&mut self) -> io::Result<()> {
        self.crlf()?;
        writeln!(self.inner, "%!PS-Adobe-2.0")
    }

    pub fn write_header_end(&mut self) -> io::Result<()> {
        self.crlf()?;
        writeln!(self.inner, "%!")
    }

    pub fn write_meta_field(&mut self, key: &str, value: &str) -> io::Result<()> {
        self.crlf()?;
        writeln!(self.inner, "%%{}: {}", key, value)
    }

    pub fn write_meta(&mut self, text: &str) -> io::Result<()> {
        self.crlf()?;
        writeln!(self.inner, "%%{}", text)
    }

    pub fn write_comment(&mut self, text: &str) -> io::Result<()> {
        self.crlf()?;
        writeln!(self.inner, "%{}", text)
    }

    fn char_space(&mut self, i: usize) -> io::Result<()> {
        if self.lc > 73 - i {
            writeln!(self.inner)?;
            self.lc = 0;
        }
        self.lc += i;
        Ok(())
    }

    pub fn arr_open(&mut self) -> io::Result<()> {
        self.char_space(1)?;
        write!(self.inner, "[")
    }

    pub fn arr_close(&mut self) -> io::Result<()> {
        self.char_space(1)?;
        write!(self.inner, "]")
    }

    pub fn arr(&mut self, f: impl Fn(&mut Self) -> io::Result<()>) -> io::Result<()> {
        self.char_space(1)?;
        write!(self.inner, "[")?;
        self.ns = false;

        f(self)?;

        self.char_space(1)?;
        write!(self.inner, "]")?;
        self.ns = false;

        Ok(())
    }

    pub fn dict(&mut self, f: impl Fn(&mut Self) -> io::Result<()>) -> io::Result<()> {
        self.need_len(3)?;
        write!(self.inner, "<< ")?;
        self.ns = true;

        f(self)?;

        self.need_len(3)?;
        write!(self.inner, " >>")?;
        self.ns = true;

        Ok(())
    }

    pub fn begin(&mut self, f: impl FnOnce(&mut Self) -> io::Result<()>) -> io::Result<()> {
        self.need_len(5)?;
        write!(self.inner, "begin")?;
        self.ns = true;
        self.lc += 5;

        f(self)?;

        self.need_len(3)?;
        write!(self.inner, "end")?;
        self.ns = true;
        self.lc += 3;

        Ok(())
    }

    pub fn seq(&mut self, f: impl Fn(&mut Self) -> io::Result<()>) -> io::Result<()> {
        self.char_space(1)?;
        write!(self.inner, "{{")?;
        self.ns = false;

        f(self)?;

        self.char_space(1)?;
        write!(self.inner, "}}")?;
        self.ns = false;

        Ok(())
    }

    pub fn write_stream(&mut self, iter: impl Iterator<Item = u8>) -> io::Result<()> {
        self.char_space(1)?;
        write!(self.inner, "<")?;
        for byte in iter {
            self.char_space(2)?;
            write!(self.inner, "{:02X}", byte)?;
        }
        self.char_space(1)?;
        write!(self.inner, ">")?;
        Ok(())
    }

    pub fn name(&mut self, name: &str) -> io::Result<()> {
        self.need_len(name.len())?;
        write!(self.inner, "{}", name)?;
        self.lc += name.len();
        self.ns = true;
        Ok(())
    }

    pub fn lit(&mut self, lit: &str) -> io::Result<()> {
        self.ns = false;
        let len = lit.len() + 1;
        self.need_len(len)?;
        write!(self.inner, "/{}", lit)?;
        self.lc += len;
        self.ns = true;
        Ok(())
    }

    pub fn bool(&mut self, val: bool) -> io::Result<()> {
        let f = format!("{}", val);
        self.need_len(f.len())?;
        write!(self.inner, "{}", f)?;
        self.lc += f.len();
        self.ns = true;
        Ok(())
    }

    pub fn double(&mut self, val: f64) -> io::Result<()> {
        let f = format!("{}", val);
        self.need_len(f.len())?;
        write!(self.inner, "{}", f)?;
        self.lc += f.len();
        self.ns = true;
        Ok(())
    }

    pub fn bytes(&mut self, buf: &[u8]) -> io::Result<()> {
        let mut len = buf.len() + 2;
        for byte in buf {
            if matches!(*byte, 40 | 41 | 92) {
                len += 1;
            }
        }

        self.need_len(len)?;
        write!(self.inner, "(")?;

        let mut off = 0;
        for i in 0..buf.len() {
            if matches!(buf[i], 40 | 41 | 92) {
                if off < i {
                    self.inner.write_all(&buf[off..i])?;
                }
                write!(self.inner, "\\{}", buf[i] as char)?;
                off = i + 1;
            }
        }
        if off < buf.len() {
            self.inner.write_all(&buf[off..])?;
        }

        write!(self.inner, ")")?;
        self.lc += len;
        Ok(())
    }

    pub fn isize(&mut self, val: isize) -> io::Result<()> {
        let f = format!("{}", val);
        self.need_len(f.len())?;
        write!(self.inner, "{}", f)?;
        self.lc += f.len();
        self.ns = true;
        Ok(())
    }

    pub fn write_usize(&mut self, val: usize) -> io::Result<()> {
        let f = format!("{}", val);
        self.need_len(f.len())?;
        write!(self.inner, "{}", f)?;
        self.lc += f.len();
        self.ns = true;
        Ok(())
    }

    pub fn ps_def(&mut self) -> io::Result<()> {
        self.name("def")
    }

    pub fn ps_put(&mut self) -> io::Result<()> {
        self.name("put")
    }

    pub fn ps_pop(&mut self) -> io::Result<()> {
        self.name("pop")
    }

    pub fn ps_where(&mut self) -> io::Result<()> {
        self.name("where")
    }

    pub fn ps_array(&mut self) -> io::Result<()> {
        self.name("array")
    }

    pub fn ps_dict(&mut self) -> io::Result<()> {
        self.name("dict")
    }

    pub fn ps_string(&mut self) -> io::Result<()> {
        self.name("string")
    }

    pub fn ps_type(&mut self) -> io::Result<()> {
        self.name("type")
    }

    pub fn ps_print(&mut self) -> io::Result<()> {
        self.name("print")
    }

    pub fn ps_copy(&mut self) -> io::Result<()> {
        self.name("copy")
    }

    pub fn ps_index(&mut self) -> io::Result<()> {
        self.name("index")
    }

    pub fn ps_roll(&mut self) -> io::Result<()> {
        self.name("roll")
    }

    pub fn ps_rotate(&mut self) -> io::Result<()> {
        self.name("rotate")
    }

    pub fn ps_definefont(&mut self) -> io::Result<()> {
        self.name("definefont")
    }

    pub fn ps_setfont(&mut self) -> io::Result<()> {
        self.name("setfont")
    }

    pub fn ps_setcachedevice(&mut self) -> io::Result<()> {
        self.name("setcachedevice")
    }

    pub fn ps_setpagedevice(&mut self) -> io::Result<()> {
        self.name("setpagedevice")
    }

    pub fn ps_imagemask(&mut self) -> io::Result<()> {
        self.name("imagemask")
    }

    pub fn ps_matrix(&mut self) -> io::Result<()> {
        self.name("matrix")
    }

    pub fn ps_currentmatrix(&mut self) -> io::Result<()> {
        self.name("currentmatrix")
    }

    pub fn ps_transform(&mut self) -> io::Result<()> {
        self.name("transform")
    }

    pub fn ps_itransform(&mut self) -> io::Result<()> {
        self.name("itransform")
    }

    pub fn ps_moveto(&mut self) -> io::Result<()> {
        self.name("moveto")
    }

    pub fn ps_rlineto(&mut self) -> io::Result<()> {
        self.name("rlineto")
    }

    pub fn ps_rmoveto(&mut self) -> io::Result<()> {
        self.name("rmoveto")
    }

    pub fn ps_newpath(&mut self) -> io::Result<()> {
        self.name("newpath")
    }

    pub fn ps_scale(&mut self) -> io::Result<()> {
        self.name("scale")
    }

    pub fn ps_fill(&mut self) -> io::Result<()> {
        self.name("fill")
    }

    pub fn ps_load(&mut self) -> io::Result<()> {
        self.name("load")
    }

    pub fn ps_get(&mut self) -> io::Result<()> {
        self.name("get")
    }

    pub fn ps_known(&mut self) -> io::Result<()> {
        self.name("known")
    }

    pub fn ps_userdict(&mut self) -> io::Result<()> {
        self.name("userdict")
    }

    pub fn ps_statusdict(&mut self) -> io::Result<()> {
        self.name("statusdict")
    }

    pub fn ps_show(&mut self) -> io::Result<()> {
        self.name("show")
    }

    pub fn ps_showpage(&mut self) -> io::Result<()> {
        self.name("showpage")
    }

    pub fn ps_length(&mut self) -> io::Result<()> {
        self.name("length")
    }

    pub fn ps_getinterval(&mut self) -> io::Result<()> {
        self.name("getinterval")
    }

    pub fn ps_ifelse(&mut self) -> io::Result<()> {
        self.name("ifelse")
    }

    pub fn ps_if(&mut self) -> io::Result<()> {
        self.name("if")
    }

    pub fn ps_for(&mut self) -> io::Result<()> {
        self.name("for")
    }

    pub fn ps_forall(&mut self) -> io::Result<()> {
        self.name("forall")
    }

    pub fn ps_exit(&mut self) -> io::Result<()> {
        self.name("exit")
    }

    pub fn ps_save(&mut self) -> io::Result<()> {
        self.name("save")
    }

    pub fn ps_restore(&mut self) -> io::Result<()> {
        self.name("restore")
    }

    pub fn ps_exch(&mut self) -> io::Result<()> {
        self.name("exch")
    }

    pub fn ps_mul(&mut self) -> io::Result<()> {
        self.name("mul")
    }

    pub fn ps_div(&mut self) -> io::Result<()> {
        self.name("div")
    }

    pub fn ps_add(&mut self) -> io::Result<()> {
        self.name("add")
    }

    pub fn ps_sub(&mut self) -> io::Result<()> {
        self.name("sub")
    }

    pub fn ps_neg(&mut self) -> io::Result<()> {
        self.name("neg")
    }

    pub fn ps_abs(&mut self) -> io::Result<()> {
        self.name("abs")
    }

    pub fn ps_round(&mut self) -> io::Result<()> {
        self.name("round")
    }

    pub fn ps_lt(&mut self) -> io::Result<()> {
        self.name("lt")
    }

    pub fn ps_le(&mut self) -> io::Result<()> {
        self.name("le")
    }

    pub fn ps_ne(&mut self) -> io::Result<()> {
        self.name("ne")
    }

    pub fn ps_eq(&mut self) -> io::Result<()> {
        self.name("eq")
    }

    pub fn ps_gsave(&mut self) -> io::Result<()> {
        self.name("gsave")
    }

    pub fn ps_grestore(&mut self) -> io::Result<()> {
        self.name("grestore")
    }
}
