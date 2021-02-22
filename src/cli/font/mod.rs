use crate::cli::opt::{Format, Options};
use ccitt_t4_t6::g42d::encode::Encoder;
use color_eyre::eyre::{self, eyre};
use eyre::Context;
use image::ImageFormat;
use sdo_ps::out::PSWriter;
use signum::{
    chsets::{
        editor::parse_eset,
        printer::{parse_ls30, parse_ps09, parse_ps24, PSet, PrinterKind},
    },
    raster::Page,
    util::{data::BIT_STRING, Buf},
};
use std::{
    io::Stdout,
    path::{Path, PathBuf},
};

pub mod cache;
pub mod ps;

use ps::write_ls30_ps_bitmap;

pub fn process_eset(
    buffer: &[u8],
    input: Option<String>,
    out: Option<PathBuf>,
) -> eyre::Result<()> {
    let (rest, eset) = match parse_eset(buffer) {
        Ok(result) => result,
        Err(e) => {
            return Err(eyre!("Failed to parse Editor Charset: \n{}", e));
        }
    };

    if !rest.is_empty() {
        println!("Unconsumed input: {:#?}", Buf(rest));
    }

    if let Some(_a) = input {
        let mut page = Page::new(100, 24);

        let mut x = 0;
        for ci in /*48..58*/ (65..66).chain(98..109) {
            let ch = &eset.chars[ci as usize];
            page.draw_echar(x, 0, ch).unwrap();
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

    Ok(())
}

fn save_as_ccitt(pset: &PSet, opt: &Options, file: &Path) -> eyre::Result<()> {
    let out_dir = if let Some(path) = &opt.out {
        path.clone()
    } else {
        let mut file_name = file.file_name().unwrap().to_os_string();
        file_name.push(".out");
        file.with_file_name(file_name)
    };
    std::fs::create_dir_all(&out_dir)?;

    for (cval, chr) in pset.chars.iter().enumerate() {
        if chr.width > 0 {
            // TODO
            let hb = chr.hbounds();

            println!("{}: {} .. {}", cval, hb.max_lead, hb.max_tail);

            let width = chr.width as usize;
            let width = width - hb.max_tail - hb.max_lead;
            let mut encoder = Encoder::new(width, &chr.bitmap);
            encoder.skip_lead = hb.max_lead;
            encoder.skip_tail = hb.max_tail;
            //encoder.debug = cval == 87;
            let contents = encoder.encode();
            let file = format!("char-{}.{}.bin", cval, width);
            let path = out_dir.join(file);
            std::fs::write(path, contents)?;
        }
    }
    Ok(())
}

fn print_pset(pset: &PSet) {
    fn print_border(w: u8) {
        print!("+");
        for _ in 0..w {
            print!("--------");
        }
        println!("+");
    }

    for glyph in &pset.chars {
        println!("+{}, {}x{}", glyph.top, glyph.width, glyph.height);
        if glyph.width > 0 {
            print_border(glyph.width);
            for row in glyph.bitmap.chunks_exact(glyph.width as usize) {
                print!("|");
                for byte in row {
                    print!("{}", &BIT_STRING[*byte as usize]);
                }
                println!("|");
            }
            print_border(glyph.width);
        }
        println!()
    }
}

fn save_pset_png(pset: &PSet, pk: PrinterKind, out: &Path) -> eyre::Result<()> {
    for (index, glyph) in pset.chars.iter().enumerate() {
        let mut page = Page::new((glyph.width as u32 + 1) * 8, 8 + pk.line_height());
        if glyph.width > 0 {
            page.draw_printer_char(4, 4, glyph)?;
        }

        let image = page.to_image();
        let name = format!("{:02x}.png", index);
        let out_path = out.join(name);
        image
            .save_with_format(out_path, ImageFormat::Png)
            .with_context(|| "Failed to save the glyph image")?;
    }
    Ok(())
}

pub fn process_ps09(buffer: &[u8], opt: &Options) -> eyre::Result<()> {
    let (rest, pset) = match parse_ps09(buffer) {
        Ok(result) => result,
        Err(e) => {
            return Err(eyre!("Failed to parse 9-needle printer charset: \n{}", e));
        }
    };

    if !rest.is_empty() {
        println!("Unconsumed input: {:#?}", Buf(rest));
    }

    match opt.format {
        Format::Png => {
            let out = opt
                .out
                .as_ref()
                .cloned()
                .unwrap_or_else(|| opt.file.with_extension("out"));
            if !out.is_dir() {
                if !out.exists() {
                    std::fs::create_dir(&out).with_context(|| "Failed to create output folder")?;
                } else {
                    return Err(eyre!("'{}' is a file", out.display()));
                }
            }
            save_pset_png(&pset, PrinterKind::Needle9, &out)?;
        }
        _ => print_pset(&pset),
    }

    Ok(())
}

pub fn process_ps24(buffer: &[u8], _opt: &Options) -> eyre::Result<()> {
    let (rest, pset) = match parse_ps24(buffer) {
        Ok(result) => result,
        Err(e) => {
            return Err(eyre!("Failed to parse 24-needle printer charset: \n{}", e));
        }
    };

    if !rest.is_empty() {
        println!("Unconsumed input: {:#?}", Buf(rest));
    }

    print_pset(&pset);

    Ok(())
}

pub fn process_ls30(buffer: &[u8], opt: &Options) -> eyre::Result<()> {
    let (rest, lset) = match parse_ls30(buffer) {
        Ok(result) => result,
        Err(e) => {
            return Err(eyre!("Failed to parse laser printer charset: \n{}", e));
        }
    };

    if !rest.is_empty() {
        println!("Unconsumed input: {:#?}", Buf(rest));
    }

    match opt.format {
        Format::DVIPSBitmapFont => {
            let mut writer: PSWriter<Stdout> = PSWriter::new();
            write_ls30_ps_bitmap("Fa", "FONT", &mut writer, &lset, None)?;
        }
        Format::CCITTT6 => {
            save_as_ccitt(&lset, opt, &opt.file)?;
        }
        _ => {
            print_pset(&lset);
        }
    }
    Ok(())
}
