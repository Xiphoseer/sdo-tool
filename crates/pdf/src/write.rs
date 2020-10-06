use std::io::{self, Write};

use chrono::{DateTime, Local};
use pdf::{
    object::PlainRef,
    primitive::{Dictionary, PdfStream, PdfString, Primitive},
};

use crate::{common::Dict, low, util::ByteCounter};

#[must_use]
pub struct PdfDict<'a, 'b> {
    first: bool,
    f: &'b mut Formatter<'a>,
}

impl<'a, 'b> PdfDict<'a, 'b> {
    fn check_first(&mut self) -> io::Result<()> {
        if self.first {
            if self.f.indent > 0 {
                writeln!(self.f.inner)?;
            }
            self.f.indent()?;
            writeln!(self.f.inner, "<<")?;
            self.first = false;
        }
        Ok(())
    }

    pub fn field(&mut self, name: &str, value: &dyn Serialize) -> io::Result<&mut Self> {
        self.check_first()?;
        self.f.indent += 2;
        self.f.indent()?;
        self.f.needs_space = write_name(name, &mut self.f.inner)?;
        value.write(&mut self.f)?;
        writeln!(self.f.inner)?;
        self.f.indent -= 2;
        Ok(self)
    }

    pub fn opt_field<X: Serialize>(
        &mut self,
        name: &str,
        field: &Option<X>,
    ) -> io::Result<&mut Self> {
        if let Some(value) = field {
            self.field(name, value)
        } else {
            Ok(self)
        }
    }

    pub fn dict_field<X: Serialize>(
        &mut self,
        name: &str,
        dict: &Dict<X>,
    ) -> io::Result<&mut Self> {
        if dict.is_empty() {
            Ok(self)
        } else {
            self.field(name, dict)
        }
    }

    pub fn dict_res_field<X: Serialize>(
        &mut self,
        name: &str,
        res: &low::Resource<Dict<X>>,
    ) -> io::Result<&mut Self> {
        match res {
            low::Resource::Ref(r) => self.field(name, r),
            low::Resource::Immediate(dict) => self.dict_field(name, dict),
        }
    }

    pub fn arr_field<X: Serialize>(&mut self, name: &str, array: &[X]) -> io::Result<&mut Self> {
        self.check_first()?;
        self.f.indent += 2;
        self.f.indent()?;
        write_name(name, &mut self.f.inner)?;

        self.f.pdf_arr().entries(array)?.finish()?;

        writeln!(self.f.inner)?;
        self.f.indent -= 2;
        Ok(self)
    }

    pub fn finish(&mut self) -> io::Result<()> {
        if self.first {
            write!(self.f.inner, "<< >>")?;
            self.f.needs_space = false;
        } else {
            self.f.indent()?;
            write!(self.f.inner, ">>")?;
            if self.f.indent == 0 {
                writeln!(self.f.inner)?;
            }
        }
        Ok(())
    }
}

#[must_use]
pub struct PdfArr<'a, 'b> {
    first: bool,
    f: &'b mut Formatter<'a>,
}

impl<'a, 'b> PdfArr<'a, 'b> {
    fn check_first(&mut self) -> io::Result<()> {
        if self.first {
            write!(self.f.inner, "[")?;
            self.first = false;
            self.f.needs_space = false;
        }
        Ok(())
    }

    pub fn entry(&mut self, value: &dyn Serialize) -> io::Result<&mut Self> {
        self.check_first()?;
        value.write(&mut self.f)?;
        Ok(self)
    }

    pub fn entries<X: Serialize>(
        &mut self,
        i: impl IntoIterator<Item = X>,
    ) -> io::Result<&mut Self> {
        for entry in i.into_iter() {
            self.entry(&entry)?;
        }
        Ok(self)
    }

    pub fn finish(&mut self) -> io::Result<()> {
        if self.first {
            write!(self.f.inner, "[]")?;
        } else {
            write!(self.f.inner, "]")?;
        }
        Ok(())
    }
}

pub struct Formatter<'a> {
    pub(super) inner: ByteCounter<&'a mut dyn Write>,
    indent: usize,
    needs_space: bool,
    pub(super) xref: Vec<Option<(usize, u16, bool)>>,
}

impl<'a> Formatter<'a> {
    pub fn new(w: &'a mut dyn Write) -> Self {
        Self {
            inner: ByteCounter::new(w),
            indent: 0,
            needs_space: false,
            xref: vec![Some((0, 65535, true))],
        }
    }

    pub fn pdf_dict(&mut self) -> PdfDict<'a, '_> {
        PdfDict {
            first: true,
            f: self,
        }
    }

    pub fn pdf_arr(&mut self) -> PdfArr<'a, '_> {
        PdfArr {
            first: true,
            f: self,
        }
    }

    pub fn pdf_stream(&mut self, data: &[u8]) -> io::Result<()> {
        writeln!(self.inner, "stream")?;
        self.inner.write_all(data)?;
        writeln!(self.inner, "endstream")?;
        Ok(())
    }

    pub fn obj(&mut self, r#ref: PlainRef, obj: &dyn Serialize) -> io::Result<()> {
        let offset = self.inner.bytes_written();
        writeln!(self.inner, "{} {} obj", r#ref.id, r#ref.gen)?;
        obj.write(self)?;
        writeln!(self.inner, "endobj")?;

        while self.xref.len() <= (r#ref.id as usize) {
            self.xref.push(None);
        }
        self.xref[r#ref.id as usize] = Some((offset, r#ref.gen, false));
        Ok(())
    }

    pub fn xref(&mut self) -> io::Result<usize> {
        let offset = self.inner.bytes_written();
        writeln!(self.inner, "xref")?;

        let mut rest = &self.xref[..];
        let mut index = 0;
        while let Some(pos) = rest.iter().position(Option::is_some) {
            rest = &rest[pos..];
            index += pos;
            let mid = rest.iter().position(Option::is_none).unwrap_or(rest.len());
            let (a, b) = rest.split_at(mid);

            writeln!(self.inner, "{} {}", index, mid)?;
            for elem in a {
                let (offset, gen, free) = elem.unwrap();
                let mark = if free { 'f' } else { 'n' };
                writeln!(self.inner, "{:010} {:05} {}", offset, gen, mark)?;
            }

            rest = b;
            index += mid;
        }

        Ok(offset)
    }

    fn indent(&mut self) -> io::Result<()> {
        write!(self.inner, "{:indent$}", "", indent = self.indent)?;
        Ok(())
    }
}

pub trait Serialize {
    fn write(&self, f: &mut Formatter) -> io::Result<()>;
}

impl<X: Serialize> Serialize for &'_ X {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        (*self).write(f)
    }
}

impl Serialize for PdfString {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.needs_space = write_string(self, &mut f.inner)?;
        Ok(())
    }
}

macro_rules! serialize_display_impl {
    ($ty:ty) => {
        impl Serialize for $ty {
            fn write(&self, f: &mut Formatter) -> io::Result<()> {
                if f.needs_space {
                    write!(f.inner, " ")?;
                }
                write!(f.inner, "{}", self)?;
                f.needs_space = true;
                Ok(())
            }
        }
    };
}

serialize_display_impl!(u8);
serialize_display_impl!(usize);
serialize_display_impl!(u32);
serialize_display_impl!(i32);
serialize_display_impl!(f32);

impl<X: Serialize> Serialize for Vec<X> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        write!(f.inner, "[")?;
        f.needs_space = false;
        for elem in self {
            elem.write(f)?;
        }
        write!(f.inner, "]")?;
        Ok(())
    }
}

impl<X: Serialize> Serialize for [X] {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        write!(f.inner, "[")?;
        f.needs_space = false;
        for elem in self {
            elem.write(f)?;
        }
        write!(f.inner, "]")?;
        Ok(())
    }
}

impl Serialize for PlainRef {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        if f.needs_space {
            write!(f.inner, " ")?;
        }
        f.needs_space = write_ref(*self, &mut f.inner)?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PdfName<'a>(pub &'a str);

impl Serialize for PdfName<'_> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.needs_space = write_name(&self.0, &mut f.inner)?;
        Ok(())
    }
}

impl Serialize for DateTime<Local> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        let off = self.offset();
        let off_sec = off.local_minus_utc();
        let (off_sec, mark) = if off_sec < 0 {
            (-off_sec, '-')
        } else {
            (off_sec, '+')
        };
        let (_, off_min) = (off_sec % 60, off_sec / 60);
        let (off_min, off_hor) = (off_min % 60, off_min / 60);
        let date_time = format!(
            "D:{}{}{:02}'{:02}",
            self.format("%Y%m%d%H%M%S"),
            mark,
            off_hor,
            off_min
        );
        let st = PdfString::new(date_time.into_bytes());
        f.needs_space = write_string(&st, &mut f.inner)?;
        Ok(())
    }
}

pub fn write_array<W: Write>(array: &[Primitive], w: &mut W) -> io::Result<bool> {
    if array.len() > 19 {
        writeln!(w, "[")?;
    } else {
        write!(w, "[")?;
    }
    for chunk in array.chunks(20) {
        let mut needs_space = false;
        for elem in chunk {
            if needs_space {
                write!(w, " ")?;
            }
            needs_space = write_primitive(elem, w)?;
        }
        if chunk.len() == 20 {
            writeln!(w)?;
        }
    }
    write!(w, "]")?;
    Ok(false)
}

pub fn write_dict<W: Write>(dict: &Dictionary, w: &mut W) -> io::Result<bool> {
    writeln!(w, "<<")?;
    for (k, v) in dict {
        write!(w, "/{} ", k)?;
        write_primitive(v, w)?;
        writeln!(w)?;
    }
    writeln!(w, ">>")?;
    Ok(false)
}

pub fn write_stream<W: Write>(stream: &PdfStream, w: &mut W) -> io::Result<bool> {
    write_dict(&stream.info, w)?;
    writeln!(w, "stream")?;
    w.write_all(&stream.data)?;
    writeln!(w, "\nendstream")?;
    Ok(true)
}

pub fn write_string<W: Write>(st: &PdfString, w: &mut W) -> io::Result<bool> {
    let bytes = st.as_bytes();

    let mut cpc = bytes.iter().copied().filter(|c| *c == 41 /* ')' */).count();
    let mut opc = 0;
    write!(w, "(")?;
    for byte in st.as_bytes() {
        match byte {
            0..=31 | 127..=255 => write!(w, "\\{:03o}", byte)?,
            92 => write!(w, "\\\\")?,
            40 => {
                if cpc == 0 {
                    write!(w, "\\(")?
                } else {
                    write!(w, "(")?;
                    cpc -= 1;
                    opc += 1;
                }
            }
            41 => {
                if opc == 0 {
                    write!(w, "\\)")?
                } else {
                    write!(w, ")")?;
                    opc -= 1;
                }
            }
            _ => write!(w, "{}", *byte as char)?,
        }
    }
    write!(w, ")")?;
    Ok(false)
}

pub fn write_name<W: Write>(name: &str, w: &mut W) -> io::Result<bool> {
    write!(w, "/{}", name)?;
    Ok(true)
}

pub fn write_ref<W: Write>(plain_ref: PlainRef, w: &mut W) -> io::Result<bool> {
    write!(w, "{} {} R", plain_ref.id, plain_ref.gen)?;
    Ok(true)
}

pub fn write_primitive<W: Write>(prim: &Primitive, w: &mut W) -> io::Result<bool> {
    match prim {
        Primitive::Null => {
            write!(w, "null")?;
            Ok(true)
        }
        Primitive::Integer(x) => {
            write!(w, "{}", x)?;
            Ok(true)
        }
        Primitive::Number(x) => {
            write!(w, "{}", x)?;
            Ok(true)
        }
        Primitive::Boolean(b) => {
            write!(w, "{}", b)?;
            Ok(true)
        }
        Primitive::String(st) => write_string(st, w),
        Primitive::Stream(stream) => write_stream(stream, w),
        Primitive::Dictionary(dict) => write_dict(dict, w),
        Primitive::Array(array) => write_array(array, w),
        Primitive::Reference(plain_ref) => write_ref(*plain_ref, w),
        Primitive::Name(name) => write_name(name, w),
    }
}
