use std::{env::args, fs::File, io, io::BufWriter, io::Write};

use ccitt_t4_t6::{
    bit_iter::{BitIter, BitWriter},
    g42d::encode::Encoder,
    g42d::{decode::Decoder, fax_decode},
};
use pdf::{
    backend::Backend,
    file::Storage,
    file::Trailer,
    object::PlainRef,
    object::{Object, Resolve, Stream},
    primitive::Dictionary,
    primitive::PdfStream,
    primitive::PdfString,
    primitive::Primitive,
};
use util::ByteCounter;
mod util;

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|substr| substr == needle)
}

fn ascii_85_encode(data: &[u8]) -> Vec<u8> {
    let new_cap = (data.len() * 6) / 4;
    let mut new_data = Vec::with_capacity(new_cap);

    let mut ctr = 0;

    let mut chunks_exact = data.chunks_exact(4);
    for group in &mut chunks_exact {
        let buf = u32::from_be_bytes([group[0], group[1], group[2], group[3]]);
        if buf == 0 {
            new_data.push(0x7A);
            ctr += 1;
        } else {
            let (c_5, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let (c_4, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let (c_3, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let (c_2, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let c_1 = buf as u8 + 33;
            new_data.extend_from_slice(&[c_1, c_2, c_3, c_4, c_5]);
            ctr += 5;
        }

        if ctr >= 75 {
            ctr = 0;
            new_data.push(10);
        }
    }
    match *chunks_exact.remainder() {
        [b_1] => {
            let buf = u32::from_be_bytes([b_1, 0, 0, 0]) / (85 * 85 * 85);
            let (c_2, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let c_1 = buf as u8 + 33;
            new_data.extend_from_slice(&[c_1, c_2, 0x7E, 0x3E]);
        }
        [b_1, b_2] => {
            let buf = u32::from_be_bytes([b_1, b_2, 0, 0]) / (85 * 85);
            let (c_3, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let (c_2, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let c_1 = buf as u8 + 33;
            new_data.extend_from_slice(&[c_1, c_2, c_3, 0x7E, 0x3E]);
        }
        [b_1, b_2, b_3] => {
            let buf = u32::from_be_bytes([b_1, b_2, b_3, 0]) / 85;
            let (c_4, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let (c_3, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let (c_2, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let c_1 = buf as u8 + 33;
            new_data.extend_from_slice(&[c_1, c_2, c_3, c_4, 0x7E, 0x3E]);
        }
        _ => {
            new_data.extend_from_slice(&[0x7E, 0x3E]);
        }
    }

    new_data
}

fn write_array<W: Write>(array: &[Primitive], w: &mut W) -> io::Result<bool> {
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

fn write_dict<W: Write>(dict: &Dictionary, w: &mut W) -> io::Result<bool> {
    writeln!(w, "<<")?;
    for (k, v) in dict {
        write!(w, "/{} ", k)?;
        write_primitive(v, w)?;
        writeln!(w)?;
    }
    writeln!(w, ">>")?;
    Ok(false)
}

fn write_stream<W: Write>(stream: &PdfStream, w: &mut W) -> io::Result<bool> {
    write_dict(&stream.info, w)?;
    writeln!(w, "stream")?;
    w.write_all(&stream.data)?;
    writeln!(w, "\nendstream")?;
    Ok(true)
}

fn write_string<W: Write>(st: &PdfString, w: &mut W) -> io::Result<bool> {
    match st.as_str() {
        Ok(s) => {
            write!(w, "(")?;
            for c in s.chars() {
                if c == '\\' || c == '(' || c == ')' {
                    write!(w, "\\{}", c)?;
                } else {
                    write!(w, "{}", c)?;
                }
            }
            write!(w, ")")?;
        }
        Err(_) => {
            write!(w, "<")?;
            for byte in st.as_bytes() {
                write!(w, "{:02X}", byte)?;
            }
            write!(w, ">")?;
        }
    }
    Ok(false)
}

fn write_primitive<W: Write>(prim: &Primitive, w: &mut W) -> io::Result<bool> {
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
        Primitive::Reference(r) => {
            write!(w, "{} {} R", r.id, r.gen)?;
            Ok(true)
        }
        Primitive::Name(name) => {
            write!(w, "/{}", name)?;
            Ok(true)
        }
    }
}

fn main() -> io::Result<()> {
    let path = args().nth(1).expect("Usage: sdo-pdf <INFILE> <OUTFILE>");
    let out = args().nth(2).expect("Usage: sdo-pdf <INFILE> <OUTFILE>");
    println!("Reading: {}", path);

    let data = std::fs::read(path).expect("could not open file");
    let (xref_tab, trailer_dict) = data.read_xref_table_and_trailer().unwrap();
    let storage = Storage::new(data, xref_tab);

    let output = File::create(out)?;
    let of = BufWriter::new(output);
    let mut of = ByteCounter::new(of);

    // 1.5, because we provide a font-descriptor for a type 3 font
    writeln!(of, "%PDF-1.5")?;
    of.write_all(&[37, 157, 164, 210, 244, 10])?;

    let mut new_trailer_dict = trailer_dict.clone();

    let trailer: Trailer = Trailer::from_dict(trailer_dict, &storage).expect("Expect `Trailer`");
    println!("trailer.highest_id: {}", trailer.highest_id);

    let mut xref = Vec::with_capacity(trailer.highest_id as usize);

    let next_ref_init = trailer.highest_id as u64;
    let mut next_ref = next_ref_init;
    let mut custom: Vec<(u64, Primitive)> = vec![];

    for id in 1..next_ref_init {
        let prim = match storage.resolve(PlainRef { id, gen: 0 }) {
            Ok(p) => p,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };

        println!("{} 0 obj", id);

        let offset = of.bytes_written();
        xref.push((id, offset));

        writeln!(of, "{} 0 obj", id)?;

        match prim {
            Primitive::Stream(pdf_stream) => {
                let new_data = ascii_85_encode(&pdf_stream.data);
                let new_len = new_data.len();

                let mut new_info = Dictionary::new();
                let mut has_filter = false;

                for (key, value) in &pdf_stream.info {
                    let new_val = match key.as_str() {
                        "Filter" => {
                            let mut filter = vec![Primitive::Name(String::from("ASCII85Decode"))];
                            match value {
                                Primitive::Array(arr) => {
                                    for elem in arr {
                                        filter.push(elem.clone());
                                    }
                                }
                                Primitive::Name(name) => {
                                    filter.push(Primitive::Name(name.clone()));
                                }
                                _ => unimplemented!(),
                            }
                            has_filter = true;
                            Primitive::Array(filter)
                        }
                        "Length" => Primitive::Integer(new_len as i32),
                        _ => value.clone(),
                    };
                    new_info.insert(key.clone(), new_val);
                }

                if !has_filter {
                    new_info.insert(
                        String::from("Filter"),
                        Primitive::Name(String::from("ASCII85Decode")),
                    );
                }

                let new_stream = PdfStream {
                    info: new_info,
                    data: new_data,
                };
                let new_stream_prim = Primitive::Stream(new_stream);
                write_primitive(&new_stream_prim, &mut of)?;

                println!("Stream({:?})", pdf_stream.info);
                let stream =
                    Stream::<()>::from_stream(pdf_stream, &storage).expect("Expected Stream");
                let decoded = stream.decode().expect("Expected valid stream");

                let mut stdout = std::io::stdout();
                println!("```stream");
                stdout.write_all(&decoded).unwrap();
                println!("```");

                let bytes = decoded.as_ref();
                if let Some(pos_a) = bytes.windows(3).position(|slice| slice == b"ID ") {
                    if let Some(pos_b) = bytes.windows(3).position(|slice| slice == b"\nEI") {
                        let start = pos_a + 3;
                        let glyph_data = &bytes[start..pos_b];
                        println!("offset: {}..{}", start, pos_b);
                        println!("{:?}", glyph_data);

                        print!("byte:");
                        for byte in glyph_data {
                            print!(" {:08b}", *byte);
                        }
                        println!();

                        if let Some(pos_f) = find(bytes, b"/F") {
                            let _pos_f = pos_f + 2;

                            let pos_c = find(bytes, b"/Columns ").expect("Expect Columns") + 9;
                            let pos_d = find(&bytes[pos_c..], b">>").expect("Expect >>") + pos_c;
                            let col_bytes = &bytes[pos_c..pos_d];
                            let col_str = std::str::from_utf8(col_bytes).unwrap();
                            let width = usize::from_str_radix(col_str, 10).unwrap();
                            println!("width: {}", width);
                            println!();
                            fax_decode(glyph_data, width);

                            let mut decoder: Decoder<BitWriter> = Decoder::new(width);
                            if let Err(e) = decoder.decode(glyph_data) {
                                println!("{:?}", e);
                            } else {
                                let res = decoder.into_store();
                                let decoded = res.done();
                                let iter = BitIter::new(&decoded);
                                iter.cli_image(width);
                                let encoder = Encoder::new(width, &decoded);
                                let done = encoder.encode();
                                
                                print!("done:");
                                for byte in done {
                                    print!(" {:08b}", byte);
                                }
                                println!();
                            }
                        }
                    }
                }
                println!();
                println!();
            }
            Primitive::Dictionary(dict) => match dict.get("Type") {
                Some(Primitive::Name(typename)) if typename.as_str() == "Font" => {
                    let mut new_dict = dict.clone();
                    new_dict.insert(
                        String::from("FontDescriptor"),
                        Primitive::Reference(PlainRef {
                            id: next_ref,
                            gen: 0,
                        }),
                    );
                    custom.push((next_ref, {
                        let mut dict = Dictionary::new();
                        let typename = String::from("FontDescriptor");
                        dict.insert(String::from("Type"), Primitive::Name(typename));
                        dict.insert(String::from("ItalicAngle"), Primitive::Integer(-20));
                        dict.insert(String::from("Flags"), Primitive::Integer(0b100));
                        let name = String::from("FUTUR_15");
                        dict.insert(String::from("FontName"), Primitive::Name(name));
                        Primitive::Dictionary(dict)
                    }));
                    next_ref += 1;

                    write_dict(&new_dict, &mut of)?;
                    println!("{:?}", new_dict);
                }
                _ => {
                    write_dict(&dict, &mut of)?;
                    println!("Dictionary({:?})", dict);
                }
            },
            _ => {
                if write_primitive(&prim, &mut of)? {
                    writeln!(of)?;
                }
                println!("{:?}", prim);
            }
        }
        writeln!(of, "endobj")?;
    }

    for (id, obj) in custom {
        let offset = of.bytes_written();
        xref.push((id, offset));
        writeln!(of, "{} 0 obj", id)?;
        if write_primitive(&obj, &mut of)? {
            writeln!(of)?;
        }
        writeln!(of, "endobj")?;
    }

    assert_eq!(next_ref as usize, xref.len() + 1);

    let startxref = of.bytes_written();
    writeln!(of, "xref")?;
    writeln!(of, "0 {}", next_ref)?;
    writeln!(of, "{:010} 65535 f", 0)?;
    for (_id, offset) in xref {
        writeln!(of, "{:010} {:05} n", offset, 0)?;
    }
    writeln!(of, "trailer")?;
    new_trailer_dict.insert(String::from("Size"), Primitive::Integer(next_ref as i32));

    write_dict(&new_trailer_dict, &mut of)?;
    writeln!(of, "startxref")?;
    writeln!(of, "{}", startxref)?;
    writeln!(of, "%%EOF")?;

    let mut buf_write = of.into_inner();
    buf_write.flush()?;
    Ok(())
}

fn _test(trailer: Dictionary, storage: Storage<Vec<u8>>) {
    println!("Trailer");
    let mut root_ref = None;
    let mut info_ref = None;

    for (key, value) in &trailer {
        println!("{}: {}", key, value);
        match key.as_str() {
            "Root" => {
                root_ref = Some(
                    value
                        .clone()
                        .to_reference()
                        .expect("Expect `Root` to be reference"),
                );
            }
            "Info" => {
                info_ref = Some(
                    value
                        .clone()
                        .to_reference()
                        .expect("Expect `Info` to be reference"),
                );
            }
            _ => {}
        }
    }
    let root_ref = root_ref.expect("Expected `Root` in trailer");
    let info_ref = info_ref.expect("Expected `Info` in trailer");
    println!("root_ref: {:?}", root_ref);
    println!("info_ref: {:?}", info_ref);

    let root = storage
        .resolve(root_ref)
        .expect("Expected `Root` reference to be valid")
        .to_dictionary(&storage)
        .expect("Expected `Root` to be a dictionary");
    let info = storage
        .resolve(info_ref)
        .expect("Expected `Info` reference to be valid")
        .to_dictionary(&storage)
        .expect("Expected `Info` to be a dictionary");
    println!("root: {:?}", root);
    println!("info: {:?}", info);

    let mut pages_ref = None;
    let mut metadata_ref = None;
    for (key, value) in &root {
        println!("{}: {}", key, value);
        match key.as_str() {
            "Pages" => {
                pages_ref = Some(
                    value
                        .clone()
                        .to_reference()
                        .expect("Expected `Pages` to be a reference"),
                );
            }
            "Metadata" => {
                metadata_ref = Some(
                    value
                        .clone()
                        .to_reference()
                        .expect("Expected `Metadata` to be a reference"),
                );
            }
            _ => {}
        }
    }

    let pages_ref = pages_ref.expect("Expected `Pages` in `Root`");
    let metadata_ref = metadata_ref.expect("Expected `Metadata` in `Root");
    println!("{:?}", pages_ref);
    println!("{:?}", metadata_ref);

    let pages = storage
        .resolve(pages_ref)
        .expect("Expected `Pages` reference to be valid")
        .to_dictionary(&storage)
        .expect("Expected `Pages` to be a dictionary");

    let metadata = storage
        .resolve(metadata_ref)
        .expect("Expected `Metadata` reference to be valid")
        .to_stream(&storage)
        .expect("Expected `Metadata` to be a dictionary");

    println!("metadata: {:?}", &metadata.info);
    println!(
        "```metadata\n{}\n```",
        std::str::from_utf8(&metadata.data).expect("Expect `Metadata` to be a valid utf-8 stream")
    );
    println!("pages: {:?}", pages);

    let mut pages_kids = None;
    for (key, value) in &pages {
        if key.as_str() == "Kids" {
            pages_kids = Some(
                value
                    .clone()
                    .to_array(&storage)
                    .expect("Expect `Pages`.`Kids` to be an array"),
            );
        }
    }

    let pages_kids = pages_kids.expect("Expect `Pages.Kids` to exist");
    for kid_ref in pages_kids {
        println!("{:?}", kid_ref);
        let kid = kid_ref
            .to_dictionary(&storage)
            .expect("Expect `Kids` entry to be a dictionary");

        println!("{:?}", kid);

        let mut contents_ref = None;
        let mut resources = None;
        for (key, value) in kid.iter() {
            match key.as_str() {
                "Contents" => {
                    contents_ref = Some(
                        value
                            .clone()
                            .to_reference()
                            .expect("Expect `Contents` to be a reference"),
                    );
                }
                "Resources" => {
                    resources = Some(
                        value
                            .clone()
                            .to_dictionary(&storage)
                            .expect("Expected `Metadata` to be a reference"),
                    );
                }
                _ => {}
            }
        }

        let resources = resources.expect("Expected `Resources` in `Page`");
        let contents_ref = contents_ref.expect("Expected `Contents` in `Page`");

        println!("resources: {:?}", resources);

        let mut ext_g_state = None;
        let mut font = None;
        for (key, value) in &resources {
            match key.as_str() {
                "Font" => {
                    font = Some(
                        value
                            .clone()
                            .to_dictionary(&storage)
                            .expect("Expect `Contents` to be a reference"),
                    );
                }
                "ExtGState" => {
                    ext_g_state = Some(
                        value
                            .clone()
                            .to_dictionary(&storage)
                            .expect("Expected `Metadata` to be a reference"),
                    );
                }
                _ => {}
            }
        }

        let ext_g_state = ext_g_state.expect("Expected `Page`.`ExtGState`");
        let font = font.expect("Expected `Page`.`Font`");

        println!("font: {}", font);

        for (key, value_ref) in &font {
            let value = value_ref
                .clone()
                .to_dictionary(&storage)
                .expect("Expect `Font` entry to be dictionary");
            println!("{}: {:#?}", key, value);

            let mut encoding = None;
            let mut to_unicode_ref = None;
            let mut char_procs = None;
            for (key, value) in &value {
                match key.as_str() {
                    "Encoding" => {
                        encoding = Some(
                            value
                                .clone()
                                .to_dictionary(&storage)
                                .expect("Expect `Encoding` to be a dictionary"),
                        );
                    }
                    "ToUnicode" => {
                        to_unicode_ref = Some(
                            value
                                .clone()
                                .to_reference()
                                .expect("Expected `ToUnicode` to be a reference"),
                        );
                    }
                    "CharProcs" => {
                        char_procs = Some(
                            value
                                .clone()
                                .to_dictionary(&storage)
                                .expect("Expected `CharProcs` to be a dictionary"),
                        );
                    }
                    _ => {}
                }
            }

            println!("to_unicode_ref: {:?}", to_unicode_ref);
            println!("char_procs: {:?}", char_procs);
            println!("encoding: {:?}", encoding);
        }

        println!("ext-g-state: {}", ext_g_state);

        println!("contents_ref: {:?}", contents_ref);

        let contents = storage
            .resolve(contents_ref)
            .expect("Expect `Contents` ref to be valid");
        let contents = contents
            .to_stream(&storage)
            .expect("Expected `Contents` to be stream");
        println!("contents.info{:?}", &contents.info);

        let content_stream =
            Stream::<()>::from_stream(contents, &storage).expect("Expect `Contents` to be valid");
        let decoded = content_stream
            .decode()
            .expect("Expect `Contents` decode to work");
        let decoded_text =
            std::str::from_utf8(&decoded).expect("Expect `Contents` to be valid utf-8");
        println!("decoded_text: {}", decoded_text);
    }
}
