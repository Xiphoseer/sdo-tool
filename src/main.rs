//! # Signum! file tool
#![warn(missing_docs)]

mod eset;
mod font;
mod sdoc;
mod util;

use sdoc::{parse_cset, parse_pbuf, parse_sdoc0001_container, parse_sysp, parse_tebu, Te, Line};
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
    let mut line_width: u16 = 0;
    let mut min_line_width  = 0xffffu16;
    let mut max_line_width = 0u16;
    for (_index, k) in data.iter().copied().enumerate() {
        match k {
            Te::Normal{
                char,
                width,
                offset,
            } => {
                if char == '\0' {
                    println!("<NUL:{}>", offset);
                    continue;
                }
                line_width += offset;
                let lcw = last_char_width.into();
                if offset >= lcw {
                    let mut space = offset - lcw;

                    // FIXME
                    if space > 250 {
                        space = 250;
                    }

                    while space >= 7 {
                        print!(" ");
                        space -= 7;
                    }
                }
                last_char_width = if char == '\n' { 0 } else { width };
                if (0xE000..=0xE080).contains(&(char as u32)) {
                    print!("<C{}>", (char as u32) - 0xE000);
                } else {
                    print!("{}", char);
                }
            }
            Te::Break(v) => {
                //println!("{{{}}}", line_width); // <br>
                println!();
                min_line_width = min_line_width.min(line_width);
                max_line_width = max_line_width.max(line_width);
                line_width = 0;
                if v > 0 {
                    print!("{})", v);
                }
            }
            Te::Paragraph(v) => {
                //println!("{{{}}}", line_width); // <br>
                println!();
                min_line_width = min_line_width.min(line_width);
                max_line_width = max_line_width.max(line_width);
                line_width = 0;
                println!();
                println!("<P>");
                if v > 0 {
                    print!("({})", v);
                }
            }
            Te::Unknown(_a) => {
                print!("<{:04X}>", _a);
            }
        }
    }
    println!();
    //println!("LINE WIDTH: ({},{})", min_line_width, max_line_width);
    let _ = (min_line_width, max_line_width);
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
                        //println!("{:#?}", part);
                    }
                    "pbuf" => {
                        let (_rest, pbuf) = parse_pbuf(part.0).unwrap();
                        println!("'pbuf': {}, {}, {}", pbuf.page_count, pbuf.kl, pbuf.first_page_nr);
                        for (page, buf) in pbuf.vec {
                            println!("  {:?}, {:?}", page, buf);
                        }
                        
                        //println!("{:#?}", Buf(rest));
                    }
                    "tebu" => {
                        let (rest, tebu) = parse_tebu(part.0).unwrap();
                        //println!("'tebu': {:?}", tebu);
                        println!("'tebu':");
                        println!("  lines_total: {}", tebu.lines_total);
                        println!("  first_page: {:?}", tebu.first_page);
                        
                        println!("------------------- [PAGE 1] -------------------");

                        for line_buf in tebu.lines {
                            //println!("SKIP: {}", line_buf.skip);
                            
                            if let Ok(line) = line_buf.parse() {
                                match line {
                                    Line::Zero(a,b) => {
                                        println!("<zero {} {} +{}>", a, b, line_buf.skip);
                                    }
                                    Line::Paragraph(data) => {
                                        println!("<p +{}>", line_buf.skip);
                                        print_tebu_data(data);
                                    }
                                    Line::Line(data) => {
                                        println!("<br +{}>", line_buf.skip);
                                        print_tebu_data(data);
                                    }
                                    Line::Line1(unknown, data) => {
                                        println!("<br' {:?} +{}>", unknown, line_buf.skip);
                                        print_tebu_data(data);
                                    }
                                    Line::FirstPageEnd => {
                                        println!("------------------- [ EOP1 ] -------------------");
                                    }
                                    Line::NewPage(page_num) => {
                                        println!("------------------- [PAGE {}] -------------------", page_num);
                                    }
                                    Line::PageEnd(page_num) => {
                                        println!("------------------- [ EOP{} ] -------------------", page_num);
                                    }
                                    Line::Unknown(u) => {
                                        println!("Unknown line kind {:?}", u);
                                        println!("SKIP: {}", line_buf.skip);
                                        println!("{:#?}", Buf(line_buf.data));
                                    }
                                };
                            }
                        }

                        /*println!("----------------------------");
                        print_tebu_data(tebu.data1);
                        println!("----------------------------");
                        print_tebu_data(tebu.data2);
                        println!("----------------------------");*/
                        println!("{:#?}", Buf(rest));
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
