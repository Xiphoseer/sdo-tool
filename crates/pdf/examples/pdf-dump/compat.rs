//! This module provides a shim to write pdf-rs primitives
use std::io;

use pdf::primitive::{Dictionary, PdfStream, Primitive};
use pdf_create::{common::ObjRef, write::{write_name, write_ref, write_string}};

/// Writes a complete PDF array to a writer
pub fn write_array<W: io::Write>(array: &[Primitive], w: &mut W) -> io::Result<bool> {
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

/// Writes a complete dict to a writer
pub fn write_dict<W: io::Write>(dict: &Dictionary, w: &mut W) -> io::Result<bool> {
    writeln!(w, "<<")?;
    for (k, v) in dict {
        write!(w, "/{} ", k)?;
        write_primitive(v, w)?;
        writeln!(w)?;
    }
    writeln!(w, ">>")?;
    Ok(false)
}

/// Writes a complete stream to a writer
pub fn write_stream<W: io::Write>(stream: &PdfStream, w: &mut W) -> io::Result<bool> {
    write_dict(&stream.info, w)?;
    writeln!(w, "stream")?;
    w.write_all(&stream.data)?;
    writeln!(w, "\nendstream")?;
    Ok(true)
}

/// Write a pdf-rs primitive
pub fn write_primitive<W: io::Write>(prim: &Primitive, w: &mut W) -> io::Result<bool> {
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
        Primitive::String(st) => write_string(st.as_bytes(), w),
        Primitive::Stream(stream) => write_stream(stream, w),
        Primitive::Dictionary(dict) => write_dict(dict, w),
        Primitive::Array(array) => write_array(array, w),
        Primitive::Reference(plain_ref) => write_ref(ObjRef {
            id: plain_ref.id,
            gen: plain_ref.gen,
        }, w),
        Primitive::Name(name) => write_name(name, w),
    }
}