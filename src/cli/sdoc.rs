use crate::{
    font::{antikro, eset::OwnedESet},
    print::Page,
    sdoc::{
        parse_cset, parse_hcim, parse_image, parse_line, parse_pbuf, parse_sdoc0001_container,
        parse_sysp, parse_tebu_header, Flags, Line, LineIter, Style, Te,
    },
    util::Buf,
    Options,
};
use anyhow::anyhow;
use image::ImageFormat;
use prettytable::{cell, format, row, Cell, Row, Table};
use std::path::Path;

fn print_tebu_data(data: Vec<Te>, chsets: &[Option<OwnedESet>; 8]) {
    let mut last_char_width: u8 = 0;
    let mut style = Style::default();

    for (_index, k) in data.iter().copied().enumerate() {
        let chr = antikro::decode(k.cval);
        if chr == '\0' {
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

        let width = if let Some(eset) = &chsets[k.cset as usize] {
            eset.chars[k.cval as usize].width
        } else {
            // default for fonts that are missing
            antikro::WIDTH[k.cval as usize]
        };
        last_char_width = if chr == '\n' { 0 } else { width };
        if (0xE000..=0xE080).contains(&(chr as u32)) {
            print!("<C{}>", (chr as u32) - 0xE000);
        } else if (0x1FBF0..=0x1FBF9).contains(&(chr as u32)) {
            print!("[{}]", chr as u32 - 0x1FBF0);
        } else {
            if k.style.underlined {
                print!("\u{0332}");
            }
            print!("{}", chr);
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

fn print_line(line: Line, skip: u16, chsets: &[Option<OwnedESet>; 8]) {
    if line.flags.contains(Flags::PAGE) {
        if line.flags.contains(Flags::PNEW) {
            println!(
                "{:04X} ------------------- [PAGE{:3}] -------------------",
                skip, line.extra
            );
        } else if line.flags.contains(Flags::PEND) {
            println!(
                "{:04X} ------------------- [EOP {:3}] -------------------",
                skip, line.extra
            );
        }
    } else {
        if line.flags.contains(Flags::FLAG) {
            println!("<F: {}>", line.extra);
        }

        if line.flags.contains(Flags::PARA) {
            print!("<p>");
        }

        print_tebu_data(line.data, chsets);

        if line.flags.contains(Flags::ALIG) {
            print!("<A>");
        }

        if line.flags.contains(Flags::LINE) {
            print!("<br>");
        }

        println!("{{{}}}", skip);
    }
}

fn process_sdoc_cset(
    part: Buf,
    chsets: &mut [Option<OwnedESet>; 8],
    _opt: &Options,
    file: &Path,
) -> anyhow::Result<()> {
    let (_, charsets) = parse_cset(part.0).unwrap();
    println!("'cset': {:?}", charsets);

    let folder = file.parent().unwrap();
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
            match page.draw_char(*x, y, ch) {
                Ok(()) => {}
                Err(()) => {
                    eprintln!("Char out of bounds {:?}", te);
                }
            }
        }
    }
    Ok(())
}

fn print_char_cmds(data: &[Te], x: &mut u16, y: u16) {
    for te in data {
        *x += te.offset;
        println!("({}, {}, {},  {}),", *x, y, te.cval, te.cset);
    }
}

struct Pos {
    x: u16,
    y: u16,
}

impl Pos {
    fn reset(&mut self) {
        *self = Self::new();
    }

    fn new() -> Self {
        Self { x: 0, y: 0 }
    }
}

fn draw_line(
    line: Line,
    skip: u16,
    page: &mut Page,
    pos: &mut Pos,
    chsets: &[Option<OwnedESet>; 8],
) -> anyhow::Result<()> {
    pos.x = 0;
    pos.y += (skip + 1) * 2;

    if line.flags.contains(Flags::FLAG) {
        println!("<F: {}>", line.extra);
    }

    if line.flags.contains(Flags::ALIG) {}

    draw_chars(&line.data, page, &mut pos.x, pos.y, chsets)?;

    Ok(())
}

fn print_line_cmds(line: Line, skip: u16, pos: &mut Pos) {
    pos.x = 0;
    pos.y += (skip + 1) * 2;

    print_char_cmds(&line.data, &mut pos.x, pos.y);
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
    let mut pos = Pos::new();
    let mut index = 0;

    for maybe_line_buf in &mut iter {
        let line_buf = match maybe_line_buf {
            Ok(line_buf) => line_buf,
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        };
        let (rest, line) = match parse_line(line_buf.data) {
            Ok(x) => x,
            Err(e) => {
                println!("Could not parse {:#?}", Buf(line_buf.data));
                println!("Error: {}", e);
                continue;
            }
        };

        if let Some(out_path) = &opt.out {
            if line.flags.contains(Flags::PAGE) {
                if line.flags.contains(Flags::PNEW) {
                    page = Page::new(750, 1120);
                } else if line.flags.contains(Flags::PEND) {
                    let image = page.to_image();
                    let file_name = format!("page-{}.png", index);
                    println!("Saving {}", file_name);
                    let page_path = out_path.join(&file_name);
                    image.save_with_format(&page_path, ImageFormat::Png)?;
                    index += 1;
                    pos.reset();
                }
            } else {
                draw_line(line, line_buf.skip, &mut page, &mut pos, chsets)?;
            }
        } else if opt.pdraw {
            print_line_cmds(line, line_buf.skip, &mut pos)
        } else {
            print_line(line, line_buf.skip, chsets);
        }

        if !rest.is_empty() {
            println!("Unconsumed line buffer rest {:#?}", Buf(rest));
        }
    }

    if !iter.rest.is_empty() {
        println!("{:#?}", Buf(iter.rest));
    }
    Ok(())
}

fn process_sdoc_hcim(part: Buf, opt: &Options) -> anyhow::Result<()> {
    let (rest, hcim) = parse_hcim(part.0).unwrap();
    println!("'hcim':");
    println!("  {:?}", hcim.header);

    let out_img = opt.imout.as_ref();
    if let Some(out_img) = out_img {
        std::fs::create_dir_all(out_img)?;
    }

    let mut image_table = Table::new();
    image_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    // Add a row per time
    image_table.set_titles(row![
        "page", "pos_x", "pos_y", "[3]", "[4]", "[5]", "sel_x", "sel_y", "sel_w", "sel_h", "[A]",
        "[B]", "[C]", "img", "[E]", "[F]",
    ]);

    for isite in hcim.sites {
        image_table.add_row(Row::new(vec![
            Cell::new(&format!("{}", isite.page)),
            Cell::new(&format!("{}", isite.pos_x)),
            Cell::new(&format!("{}", isite.pos_y)),
            Cell::new(&format!("{}", isite._3)),
            Cell::new(&format!("{}", isite._4)),
            Cell::new(&format!("{}", isite._5)),
            Cell::new(&format!("{}", isite.sel_x)),
            Cell::new(&format!("{}", isite.sel_y)),
            Cell::new(&format!("{}", isite.sel_w)),
            Cell::new(&format!("{}", isite.sel_h)),
            Cell::new(&format!("{}", isite._A)),
            Cell::new(&format!("{}", isite._B)),
            Cell::new(&format!("{}", isite._C)),
            Cell::new(&format!("{}", isite.img)),
            Cell::new(&format!("{}", isite._E)),
            Cell::new(&format!("{:?}", isite._F)),
        ]));
    }

    image_table.printstd();

    for (index, img) in hcim.images.iter().enumerate() {
        println!("image[{}]:", index);
        match parse_image(img.0) {
            Ok((_imgrest, im)) => {
                println!("IMAGE: {:?}", im.key);
                println!("{:#?}", im.bytes);
                if let Some(out_img) = out_img.as_ref() {
                    let name = format!("{:02}-{}.png", index, im.key);
                    let path = out_img.join(name);
                    let page = Page::from_screen(im.image);
                    let img = page.to_image();
                    img.save_with_format(&path, ImageFormat::Png)?;
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    if !rest.is_empty() {
        println!("{:#?}", Buf(rest));
    }

    Ok(())
}

pub fn process_sdoc(buffer: &[u8], opt: Options, file: &Path) -> anyhow::Result<()> {
    let (rest, sdoc) = match parse_sdoc0001_container(buffer) {
        Ok(x) => x,
        Err(nom::Err::Failure((rest, kind))) => {
            return Err(anyhow!("Parse failed [{:?}]:\n{:?}", rest, kind));
        }
        Err(nom::Err::Error((rest, kind))) => {
            return Err(anyhow!("Parse errored [{:?}]:\n{:?}", rest, kind));
        }
        Err(nom::Err::Incomplete(a)) => {
            return Err(anyhow!("Parse incomplete, needed {:?}", a));
        }
    };
    let mut chsets: [Option<OwnedESet>; 8] = [None, None, None, None, None, None, None, None];

    if let Some(out_path) = &opt.out {
        std::fs::create_dir_all(out_path)?;
    }

    for (key, part) in sdoc.parts {
        match key {
            "cset" => {
                process_sdoc_cset(part, &mut chsets, &opt, file)?;
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
                process_sdoc_hcim(part, &opt)?;
            }
            _ => {
                println!("'{}': {}", key, part.0.len());
            }
        }
    }

    if !rest.is_empty() {
        println!("remaining: {:#?}", Buf(rest));
    }
    Ok(())
}
