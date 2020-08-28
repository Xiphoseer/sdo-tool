//! # Signum! file tool
#![warn(missing_docs)]

mod eset;
mod font;
mod sdoc;
mod util;

use sdoc::{
    parse_cset, parse_line, parse_pbuf, parse_sdoc0001_container, parse_sysp, parse_tebu_header,
    Line, LineIter, Style, Te,
};
use util::Buf;

use anyhow::anyhow;
use eset::parse_eset;
use nom::Err;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    /// A file to process
    file: PathBuf,
    /// HACK: decode atari document to utf8
    #[structopt(long)]
    decode: bool,
}

fn process_eset(buffer: &[u8]) -> anyhow::Result<()> {
    match parse_eset(buffer) {
        Ok((_rest, eset)) => {
            assert!(_rest.is_empty());
            println!("{:?}", eset.buf1);
            eset.print();
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
    Ok(())
}

fn print_tebu_data(data: Vec<Te>) {
    let mut last_char_width: u8 = 0;
    let mut style = Style::default();

    for (_index, k) in data.iter().copied().enumerate() {
        if k.char == '\0' {
            println!("<NUL:{}>", k.offset);
            continue;
        }

        if !k.style.bold && style.bold {
            style.bold = false;
            print!("</b>");
        }
        if !k.style.italic && style.italic {
            style.italic = false;
            print!("</i>");
        }
        if !k.style.sth2 && style.sth2 {
            style.sth2 = false;
            print!("</sth2>");
        }
        if !k.style.sth1 && style.sth1 {
            style.sth1 = false;
            print!("</sth1>");
        }

        let lcw = last_char_width.into();
        if k.offset >= lcw {
            let mut space = k.offset - lcw;

            while space >= 7 {
                print!(" ");
                space -= 7;
            }
        }
        last_char_width = if k.char == '\n' { 0 } else { k.width };
        if (0xE000..=0xE080).contains(&(k.char as u32)) {
            print!("<C{}>", (k.char as u32) - 0xE000);
        } else {
            if k.style.footnote {
                print!("<footnote>");
            }

            if k.style.small {
                print!("<small>");
            }

            if k.style.sth1 && !style.sth1 {
                style.sth1 = true;
                print!("<sth1>");
            }
            if k.style.sth2 && !style.sth2 {
                style.sth2 = true;
                print!("<sth2>");
            }
            if k.style.italic && !style.italic {
                style.italic = true;
                print!("<i>");
            }
            if k.style.bold && !style.bold {
                style.bold = true;
                print!("<b>");
            }

            if k.style.underlined {
                print!("\u{0332}");
            }
            print!("{}", k.char);
        }
    }
    if style.bold {
        print!("</b>");
    }
    if style.italic {
        print!("</i>");
    }
    if style.sth2 {
        print!("</sth2>");
    }
    if style.sth1 {
        print!("</sth1>");
    }
}

fn process_sdoc(buffer: &[u8]) -> anyhow::Result<()> {
    match parse_sdoc0001_container(&buffer) {
        Ok((rest, sdoc)) => {
            for (key, part) in sdoc.parts {
                match key {
                    "cset" => {
                        let (_, charsets) = parse_cset(part.0).unwrap();
                        println!("'cset': {:?}", charsets);
                    }
                    "sysp" => {
                        let (_, sysp) = parse_sysp(part.0).unwrap();
                        println!("'sysp': {:#?}", sysp);
                    }
                    "pbuf" => {
                        let (_rest, pbuf) = parse_pbuf(part.0).unwrap();
                        println!(
                            "'pbuf': {}, {}, {}",
                            pbuf.page_count, pbuf.kl, pbuf.first_page_nr
                        );
                        for (page, buf) in pbuf.vec {
                            println!("  {:?}, {:?}", page, buf);
                        }
                    }
                    "tebu" => {
                        let (rest, tebu_header) = parse_tebu_header(part.0).unwrap();
                        //println!("'tebu': {:?}", tebu);
                        println!("'tebu': {:?}", tebu_header);

                        let mut iter = LineIter(rest);

                        println!("------------------- [PAGE 1] -------------------");

                        for maybe_line_buf in &mut iter {
                            let line_buf = match maybe_line_buf {
                                Ok(line_buf) => line_buf,
                                Err(e) => return Err(anyhow!("{}", e)),
                            };
                            match parse_line(line_buf.data) {
                                Ok((rest, line)) => {
                                    match line {
                                        Line::Zero(data) => {
                                            println!("<zero +{}>", line_buf.skip);
                                            print_tebu_data(data);
                                            println!();
                                        }
                                        Line::Paragraph(data) => {
                                            println!("<p +{}>", line_buf.skip);
                                            print_tebu_data(data);
                                            println!();
                                        }
                                        Line::Paragraph1(unknown, data) => {
                                            println!("<p' {:?} +{}>", unknown, line_buf.skip);
                                            print_tebu_data(data);
                                            println!();
                                        }
                                        Line::Line(data) => {
                                            println!("<br +{}>", line_buf.skip);
                                            print_tebu_data(data);
                                            println!();
                                        }
                                        Line::Line1(unknown, data) => {
                                            println!("<br' {:?} +{}>", unknown, line_buf.skip);
                                            print_tebu_data(data);
                                            println!();
                                        }
                                        Line::Heading(data) => {
                                            print!("<h1 +{}>", line_buf.skip);
                                            let newlines = !data.is_empty();
                                            if newlines {
                                                println!();
                                            }
                                            print_tebu_data(data);
                                            if newlines {
                                                println!();
                                            }
                                        }
                                        Line::Some(data) => {
                                            println!("<s +{}>", line_buf.skip);
                                            print_tebu_data(data);
                                            println!();
                                        }
                                        Line::Heading2(data) => {
                                            println!("<h2 +{}>", line_buf.skip);
                                            print_tebu_data(data);
                                            println!();
                                        }
                                        Line::FirstPageEnd => {
                                            println!(
                                                "------------------- [ EOP1 ] -------------------"
                                            );
                                        }
                                        Line::NewPage(page_num) => {
                                            println!(
                                                "------------------- [PAGE {}] -------------------",
                                                page_num
                                            );
                                        }

                                        Line::PageEnd(page_num) => {
                                            println!(
                                                "------------------- [ EOP{} ] -------------------",
                                                page_num
                                            );
                                        }
                                        Line::Unknown(u) => {
                                            println!("Unknown line kind {:?}", u);
                                            println!("SKIP: {}", line_buf.skip);
                                            println!("{:#?}", Buf(line_buf.data));
                                        }
                                    };

                                    if !rest.is_empty() {
                                        println!("Unconsumed line buffer rest {:#?}", Buf(rest));
                                    }
                                }
                                Err(e) => {
                                    println!("Could not parse {:#?}", Buf(line_buf.data));
                                    println!("Error: {}", e);
                                }
                            }
                        }

                        /*println!("----------------------------");
                        print_tebu_data(tebu.data1);
                        println!("----------------------------");
                        print_tebu_data(tebu.data2);
                        println!("----------------------------");*/
                        println!("{:#?}", Buf(iter.0));
                    }
                    _ => {
                        println!("'{}': {}", key, part.0.len());
                    }
                }
            }
            println!("remaining: {:?}", rest.len());
        }
        Err(Err::Failure((rest, kind))) => {
            return Err(anyhow!("Parse failed [{:?}]:\n{:?}", rest, kind));
        }
        Err(Err::Error((rest, kind))) => {
            return Err(anyhow!("Parse errored [{:?}]:\n{:?}", rest, kind));
        }
        Err(Err::Incomplete(a)) => {
            return Err(anyhow!("Parse incomplete, needed {:?}", a));
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let opt = Options::from_args();

    let file = File::open(&opt.file)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();

    reader.read_to_end(&mut buffer)?;

    println!("\u{0308}\u{1D400}");

    if opt.decode {
        let mut decoded = String::with_capacity(buffer.len());
        for byte in buffer {
            let ch = font::decode_atari(byte);
            decoded.push(ch);
        }
        print!("{}", decoded);
        Ok(())
    } else {
        match buffer.get(..4) {
            Some(b"sdoc") => process_sdoc(&buffer),
            Some(b"eset") => process_eset(&buffer),
            Some(t) => Err(anyhow!("Unknown file type {:?}", t)),
            None => Err(anyhow!("File has less than 4 bytes")),
        }
    }
}
