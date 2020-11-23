use ccitt_t4_t6::{
    bit_iter::{BitIter, BitWriter},
    g42d::encode::Encoder,
    g42d::{decode::Decoder, fax_decode},
};

use io::{BufWriter, Write};
use pdf::{
    backend::Backend,
    file::Storage,
    file::Trailer,
    object::{Object, PlainRef, Resolve, Stream},
    primitive::Dictionary,
    primitive::PdfStream,
    primitive::Primitive,
};
use pdf_create::{
    encoding::ascii_85_encode, util::ByteCounter, write::write_dict, write::write_primitive,
};

use std::{env::args, fs::File, io};

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|substr| substr == needle)
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
                let mut new_data = Vec::with_capacity(pdf_stream.data.len() * 5 / 4);
                let new_len = ascii_85_encode(&pdf_stream.data, &mut new_data)?;

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

    //assert_eq!(next_ref as usize, xref.len() + 1);

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
