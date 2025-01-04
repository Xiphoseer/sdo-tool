//! Methods to produce a binary file

use std::io::{self, Write};

use chrono::{DateTime, Local};

use crate::{
    common::{Dict, ObjRef, PdfString},
    low,
    util::ByteCounter,
};

/// API to serialize a dict
#[must_use]
pub struct PdfDict<'a, 'b> {
    first: bool,
    f: &'b mut Formatter<'a>,
}

impl PdfDict<'_, '_> {
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

    /// Write a field
    pub fn field(&mut self, name: &str, value: &dyn Serialize) -> io::Result<&mut Self> {
        self.check_first()?;
        self.f.indent += 2;
        self.f.indent()?;
        self.f.needs_space = write_name(name, &mut self.f.inner)?;
        value.write(self.f)?;
        writeln!(self.f.inner)?;
        self.f.indent -= 2;
        Ok(self)
    }

    /// Write a field
    pub fn default_field<X: Default + PartialEq + Serialize>(
        &mut self,
        name: &str,
        value: &X,
    ) -> io::Result<&mut Self> {
        if *value != Default::default() {
            self.field(name, value)
        } else {
            Ok(self)
        }
    }

    /// Write flattened
    pub fn embed<X: ToDict>(&mut self, embed: &X) -> io::Result<&mut Self> {
        embed.write(self)?;
        Ok(self)
    }

    /// Write an optional field, if it is not `None`
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

    /// Write a dict-valued field if it is not empty
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

    /// Write a dict-valued field wrapped in a resource
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

    /// Write a slice-valued field
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

    /// Write a slice-valued field, skip if empty
    pub fn opt_arr_field<X: Serialize>(
        &mut self,
        name: &str,
        array: &[X],
    ) -> io::Result<&mut Self> {
        if array.is_empty() {
            Ok(self)
        } else {
            self.arr_field(name, array)
        }
    }

    /// Close the dict
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

/// API to serialize a type into a dict
pub trait ToDict {
    /// Add the key to the dict
    fn write(&self, dict: &mut PdfDict<'_, '_>) -> io::Result<()>;
}

/// API to serialize an array
#[must_use]
pub struct PdfArr<'a, 'b> {
    first: bool,
    f: &'b mut Formatter<'a>,
}

impl PdfArr<'_, '_> {
    fn check_first(&mut self) -> io::Result<()> {
        if self.first {
            write!(self.f.inner, "[")?;
            self.first = false;
            self.f.needs_space = false;
        }
        Ok(())
    }

    /// Write the next entry
    pub fn entry<S: Serialize>(&mut self, value: &S) -> io::Result<&mut Self> {
        self.check_first()?;
        value.write(self.f)?;
        Ok(self)
    }

    /// Write entries from an iterator
    pub fn entries<X: Serialize>(
        &mut self,
        i: impl IntoIterator<Item = X>,
    ) -> io::Result<&mut Self> {
        for entry in i.into_iter() {
            self.entry(&entry)?;
        }
        Ok(self)
    }

    /// Close the array
    pub fn finish(&mut self) -> io::Result<()> {
        if self.first {
            write!(self.f.inner, "[]")?;
        } else {
            write!(self.f.inner, "]")?;
        }
        Ok(())
    }
}

/// Formatter for a PDF document
pub struct Formatter<'a> {
    pub(super) inner: ByteCounter<&'a mut dyn Write>,
    indent: usize,
    needs_space: bool,
    pub(super) xref: Vec<Option<(usize, u16, bool)>>,
}

impl<'a> Formatter<'a> {
    /// Create a new formatter
    pub fn new(w: &'a mut dyn Write) -> Self {
        Self {
            inner: ByteCounter::new(w),
            indent: 0,
            needs_space: false,
            xref: vec![Some((0, 65535, true))],
        }
    }

    /// Start writing a PDF dict
    pub fn pdf_dict(&mut self) -> PdfDict<'a, '_> {
        PdfDict {
            first: true,
            f: self,
        }
    }

    /// Start writing a PDF array
    pub fn pdf_arr(&mut self) -> PdfArr<'a, '_> {
        PdfArr {
            first: true,
            f: self,
        }
    }

    /// Start writing a stream
    pub fn pdf_stream(&mut self, data: &[u8]) -> io::Result<()> {
        writeln!(self.inner, "stream")?;
        self.inner.write_all(data)?;
        if !data.ends_with(&[0x0a]) {
            writeln!(self.inner)?;
        }
        writeln!(self.inner, "endstream")?;
        Ok(())
    }

    /// Start writing an object
    pub fn obj(&mut self, r#ref: ObjRef, obj: &dyn Serialize) -> io::Result<()> {
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

    /// Write a classic xref section
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
                // NOTE: the PDF spec requires the eol to be two bytes long (i.e. SP LF or CR LF)
                writeln!(self.inner, "{:010} {:05} {} ", offset, gen, mark)?;
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

/// Trait to serialize some PDF object
pub trait Serialize {
    /// Write the object to a stream
    fn write(&self, f: &mut Formatter) -> io::Result<()>;
}

impl<X: Serialize> Serialize for &'_ X {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        (*self).write(f)
    }
}

impl Serialize for PdfString {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.needs_space = write_string(self.as_bytes(), &mut f.inner)?;
        Ok(())
    }
}

impl Serialize for md5::Digest {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.needs_space = false;
        write!(f.inner, "<{:?}>", self)
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
serialize_display_impl!(bool);

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

impl Serialize for ObjRef {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        if f.needs_space {
            write!(f.inner, " ")?;
        }
        f.needs_space = write_ref(*self, &mut f.inner)?;
        Ok(())
    }
}

/// A borrowed PDF name (e.g. `/Info`)
#[derive(Debug, Copy, Clone)]
pub struct PdfName<'a>(pub &'a str);

impl Serialize for PdfName<'_> {
    fn write(&self, f: &mut Formatter) -> io::Result<()> {
        f.needs_space = write_name(self.0, &mut f.inner)?;
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
        let st = date_time.into_bytes();
        f.needs_space = write_string(&st, &mut f.inner)?;
        Ok(())
    }
}

/// Writes a complete string to a writer
pub fn write_string<W: Write>(bytes: &[u8], w: &mut W) -> io::Result<bool> {
    let mut cpc = bytes.iter().copied().filter(|c| *c == 41 /* ')' */).count();
    let mut opc = 0;
    write!(w, "(")?;
    for byte in bytes.iter().copied() {
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
            _ => write!(w, "{}", byte as char)?,
        }
    }
    write!(w, ")")?;
    Ok(false)
}

/// Write a borrowed string as a PDF name
///
/// FIXME: Probably not all unicode strings allowed
pub fn write_name<W: Write>(name: &str, w: &mut W) -> io::Result<bool> {
    write!(w, "/{}", name)?;
    Ok(true)
}

/// Write a plain reference
pub fn write_ref<W: Write>(plain_ref: ObjRef, w: &mut W) -> io::Result<bool> {
    write!(w, "{} {} R", plain_ref.id, plain_ref.gen)?;
    Ok(true)
}
