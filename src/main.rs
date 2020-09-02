//! # Signum! file tool
#![warn(missing_docs)]

mod font;
mod images;
mod print;
mod sdoc;
mod util;

use sdoc::{
    parse_cset, parse_hcim, parse_image, parse_line, parse_pbuf, parse_sdoc0001_container,
    parse_sysp, parse_tebu_header, Line, LineIter, Style, Te,
};
use util::Buf;

use anyhow::anyhow;
use font::eset::{parse_eset, OwnedESet};
use image::ImageFormat;
use images::imc::parse_imc;
use nom::Err;
use prettytable::{cell, format, row, Cell, Row, Table};
use print::Page;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    /// A file to process
    file: PathBuf,
    /// HACK: decode atari document to utf8
    #[structopt(long)]
    decode: bool,
    /// Where to store the output, if applicable
    #[structopt(long)]
    out: Option<PathBuf>,
    /// Some input to process
    #[structopt(long)]
    input: Option<String>,
}

fn process_eset(buffer: &[u8], input: Option<String>, out: Option<PathBuf>) -> anyhow::Result<()> {
    match parse_eset(buffer) {
        Ok((_rest, eset)) => {
            assert!(_rest.is_empty());
            if let Some(_a) = input {
                let mut page = Page::new(100, 24);

                let mut x = 0;
                for ci in /*48..58*/ (65..66).chain(98..109) {
                    let ch = &eset.chars[ci as usize];
                    page.draw_char(x, 0, ch);
                    x += u16::from(ch.width) + 1;
                }

                if let Some(out_path) = out {
                    let image = page.to_image();
                    image.save_with_format(out_path, ImageFormat::Png)?;
                } else {
                    page.print();
                }
            } else {
                println!("{:#?}", eset.buf1);
                eset.print();
            }
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
    Ok(())
}

fn process_bimc(buffer: &[u8], out: Option<PathBuf>) -> anyhow::Result<()> {
    let decoded = parse_imc(&buffer) //
        .map_err(|err| anyhow!("Failed to parse: {}", err))?;

    let page = Page::from_screen(decoded) //
        .map_err(|x| anyhow!("Deoder produced buffer of size {}, expected 32000", x.len()))?;

    if let Some(out_path) = out {
        let image = page.to_image();
        image.save_with_format(out_path, ImageFormat::Png)?;
    } else {
        println!("Decoded image sucessfully, to store it as PNG, pass `--out <PATH>`");
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
        if !k.style.small && style.small {
            style.small = false;
            print!("</small>");
        }

        let lcw = last_char_width.into();
        if k.offset >= lcw {
            let mut space = k.offset - lcw;

            while space >= 7 {
                print!(" ");
                space -= 7;
            }
        }

        if k.style.footnote {
            print!("<footnote>");
        }
        if k.style.small && !style.small {
            style.small = true;
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

        last_char_width = if k.char == '\n' { 0 } else { k.width };
        if (0xE000..=0xE080).contains(&(k.char as u32)) {
            print!("<C{}>", (k.char as u32) - 0xE000);
        } else if (0x1FBF0..=0x1FBF9).contains(&(k.char as u32)) {
            print!("[{}]", k.char as u32 - 0x1FBF0);
        } else {
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
    if style.small {
        print!("</small>");
    }
}

fn print_line(line: Line, skip: u16) {
    match line {
        Line::Zero(data) => {
            println!("<zero +{}>", skip);
            print_tebu_data(data);
            println!();
        }
        Line::Paragraph(data) => {
            println!("<p +{}>", skip);
            print_tebu_data(data);
            println!();
        }
        Line::Paragraph1(unknown, data) => {
            println!("<p' {:?} +{}>", unknown, skip);
            print_tebu_data(data);
            println!();
        }
        Line::Line(data) => {
            println!("<br +{}>", skip);
            print_tebu_data(data);
            println!();
        }
        Line::Line1(unknown, data) => {
            println!("<br' {:?} +{}>", unknown, skip);
            print_tebu_data(data);
            println!();
        }
        Line::P800(data) => {
            println!("<p800 +{}>", skip);
            print_tebu_data(data);
            println!();
        }
        Line::Heading(data) => {
            print!("<h1 +{}>", skip);
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
            println!("<s +{}>", skip);
            print_tebu_data(data);
            println!();
        }
        Line::Heading2(data) => {
            println!("<h2 +{}>", skip);
            print_tebu_data(data);
            println!();
        }
        Line::FirstPageEnd => {
            println!(
                "{:04X} ------------------- [ EOP1 ] -------------------",
                skip
            );
        }
        Line::PageEnd(page_num) => {
            println!(
                "{:04X} ------------------- [ EOP{} ] -------------------",
                skip, page_num
            );
        }
        Line::FirstNewPage => {
            println!(
                "{:04X} ------------------- [PAGE 1] -------------------",
                skip
            );
        }
        Line::NewPage(page_num) => {
            println!(
                "{:04X} ------------------- [PAGE {}] -------------------",
                skip, page_num
            );
        }
        Line::Unknown(u) => {
            println!("Unknown line kind {:?}", u);
            println!("SKIP: {}", skip);
        }
    };
}

fn process_sdoc_cset(
    part: Buf,
    chsets: &mut [Option<OwnedESet>; 8],
    opt: &Options,
) -> anyhow::Result<()> {
    let (_, charsets) = parse_cset(part.0).unwrap();
    println!("'cset': {:?}", charsets);

    let folder = opt.file.parent().unwrap();
    let default_cset_folder = folder.join("CHSETS");

    for (index, name) in charsets.into_iter().enumerate() {
        if name.is_empty() {
            continue;
        }
        let mut editor_cset_file = default_cset_folder.join(name.as_ref());
        editor_cset_file.set_extension("E24");

        if !editor_cset_file.exists() {
            eprintln!("Font file '{}' not found", editor_cset_file.display());
            continue;
        }

        match OwnedESet::load(&editor_cset_file) {
            Ok(eset) => {
                chsets[index] = Some(eset);
                println!("Loaded font file '{}'", editor_cset_file.display());
            }
            Err(e) => {
                eprintln!("Failed to parse font file {}", editor_cset_file.display());
                eprintln!("Are you sure this is a valid Signum! editor font?");
                eprintln!("Error: {}", e);
            }
        }
    }
    Ok(())
}

fn process_sdoc_pbuf(part: Buf) -> anyhow::Result<()> {
    let (rest, pbuf) = parse_pbuf(part.0).unwrap();

    println!(
        "Page Buffer ('pbuf')\n  page_count: {}\n  kl: {}\n  first_page_nr: {}",
        pbuf.page_count, pbuf.kl, pbuf.first_page_nr
    );

    if !rest.is_empty() {
        println!("  rest: {:+?}", Buf(rest));
    }

    // Create the table
    let mut page_table = Table::new();
    page_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    // Add a row per time
    page_table.set_titles(row![
        "idx", "#phys", "#log", "lines", "m-l", "m-r", "m-t", "m-b", "numbpos", "kapitel",
        "intern", "rest",
    ]);

    for (page, buf) in pbuf.vec {
        page_table.add_row(Row::new(vec![
            Cell::new(&format!("{:3}", page.index)),
            Cell::new(&format!("{:5}", page.phys_pnr)),
            Cell::new(&format!("{:4}", page.log_pnr)),
            Cell::new(&format!("{:3} {:3}", page.lines.0, page.lines.1)),
            Cell::new(&format!("{:3}", page.margin.left)),
            Cell::new(&format!("{:3}", page.margin.right)),
            Cell::new(&format!("{:3}", page.margin.top)),
            Cell::new(&format!("{:3}", page.margin.bottom)),
            Cell::new(&format!("{:3} {:3}", page.numbpos.0, page.numbpos.1)),
            Cell::new(&format!("{:3} {:3}", page.kapitel.0, page.kapitel.1)),
            Cell::new(&format!("{:3} {:3}", page.intern.0, page.intern.1)),
            Cell::new(&format!("{:?}", buf)),
        ]));
    }

    // Print the table to stdout
    page_table.printstd();

    Ok(())
}

fn draw_chars(
    data: &[Te],
    page: &mut Page,
    x: &mut u16,
    y: u16,
    chsets: &[Option<OwnedESet>; 8],
) -> anyhow::Result<()> {
    for te in data {
        *x += te.offset;
        if let Some(eset) = &chsets[te.cset as usize] {
            let ch = &eset.chars[te.cval as usize];
            page.draw_char(*x, y, ch)
        }
    }
    Ok(())
}

fn draw_line(
    line: Line,
    path: &Path,
    page: &mut Page,
    x: &mut u16,
    y: &mut u16,
    index: &mut u32,
    chsets: &[Option<OwnedESet>; 8],
) -> anyhow::Result<()> {
    match line {
        Line::Zero(data) => {
            draw_chars(&data, page, x, *y, chsets)?;
            *x = 0;
        }
        Line::Paragraph(data) => {
            draw_chars(&data, page, x, *y, chsets)?;
            *x = 0;
            *y += 8;
        }
        Line::Paragraph1(_unknown, data) => {
            draw_chars(&data, page, x, *y, chsets)?;
            *x = 0;
        }
        Line::Line(data) => {
            draw_chars(&data, page, x, *y, chsets)?;
            *x = 0;
            *y += 4;
        }
        Line::Line1(_unknown, data) => {
            draw_chars(&data, page, x, *y, chsets)?;
            *x = 0;
        }
        Line::P800(data) => {
            draw_chars(&data, page, x, *y, chsets)?;
            *x = 0;
        }
        Line::Heading(data) => {
            draw_chars(&data, page, x, *y, chsets)?;
        }
        Line::Some(data) => {
            draw_chars(&data, page, x, *y, chsets)?;
            *x = 0;
        }
        Line::Heading2(data) => {
            draw_chars(&data, page, x, *y, chsets)?;
        }
        Line::FirstPageEnd | Line::PageEnd(_) => {
            let image = page.to_image();
            let file_name = format!("page-{}.png", *index);
            println!("Saving {}", file_name);
            let page_path = path.join(&file_name);
            image.save_with_format(&page_path, ImageFormat::Png)?;
            *index += 1;
            *y = 0;
        }
        Line::FirstNewPage | Line::NewPage(_) => {
            *page = Page::new(750, 800);
        }
        Line::Unknown(u) => {
            println!("Unknown line kind {:?}", u);
        }
    };

    Ok(())
}

fn process_sdoc_tebu(
    part: Buf,
    chsets: &[Option<OwnedESet>; 8],
    opt: &Options,
) -> anyhow::Result<()> {
    let (rest, tebu_header) = parse_tebu_header(part.0).unwrap();
    println!("'tebu': {:?}", tebu_header);

    let mut iter = LineIter { rest };

    let mut page = Page::new(1, 1);
    let mut x = 0;
    let mut y = 0;
    let mut index = 0;

    for maybe_line_buf in &mut iter {
        let line_buf = match maybe_line_buf {
            Ok(line_buf) => line_buf,
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        };
        match parse_line(line_buf.data) {
            Ok((rest, line)) => {
                if let Some(out_path) = &opt.out {
                    y += line_buf.skip & 0x03FF;
                    draw_line(
                        line, out_path, &mut page, &mut x, &mut y, &mut index, chsets,
                    )?;
                } else {
                    print_line(line, line_buf.skip);
                }
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

    if !iter.rest.is_empty() {
        println!("{:#?}", Buf(iter.rest));
    }
    Ok(())
}

fn process_sdoc(buffer: &[u8], opt: Options) -> anyhow::Result<()> {
    match parse_sdoc0001_container(buffer) {
        Ok((rest, sdoc)) => {
            let mut chsets: [Option<OwnedESet>; 8] =
                [None, None, None, None, None, None, None, None];

            if let Some(out_path) = &opt.out {
                std::fs::create_dir_all(out_path)?;
            }

            for (key, part) in sdoc.parts {
                match key {
                    "cset" => {
                        process_sdoc_cset(part, &mut chsets, &opt)?;
                    }
                    "sysp" => {
                        let (_, sysp) = parse_sysp(part.0).unwrap();
                        println!("'sysp': {:#?}", sysp);
                    }
                    "pbuf" => {
                        process_sdoc_pbuf(part)?;
                    }
                    "tebu" => {
                        process_sdoc_tebu(part, &chsets, &opt)?;
                    }
                    "hcim" => {
                        let (rest, hcim) = parse_hcim(part.0).unwrap();
                        println!("'hcim':");
                        println!("  {:?}", hcim.header);

                        for res_ref in hcim.ref_iter() {
                            let iref = res_ref.unwrap();
                            println!("IREF: {:?}", iref);
                        }

                        for (index, img) in hcim.images.iter().enumerate() {
                            println!("image[{}]:", index);
                            let (_imgrest, im) = parse_image(img.0).unwrap();
                            println!("{:?}", im);
                            let _name = im.key.to_string();
                            //std::fs::write(&name, &img.0).unwrap();

                            //println!("{:#?}", Buf(imgrest));
                        }

                        if !rest.is_empty() {
                            println!("{:#?}", Buf(rest));
                        }
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
            Some(b"sdoc") => process_sdoc(&buffer, opt),
            Some(b"eset") => process_eset(&buffer, opt.input, opt.out),
            Some(b"bimc") => process_bimc(&buffer, opt.out),
            Some(t) => Err(anyhow!("Unknown file type {:?}", t)),
            None => Err(anyhow!("File has less than 4 bytes")),
        }
    }
}
