use crate::{
    font::{
        editor::parse_eset,
        printer::{parse_ls30, parse_ps24},
    },
    print::Page,
    util::{data::BIT_STRING, Buf},
    Options,
};
use anyhow::anyhow;
use image::ImageFormat;
use std::path::PathBuf;

pub fn process_eset(
    buffer: &[u8],
    input: Option<String>,
    out: Option<PathBuf>,
) -> anyhow::Result<()> {
    let (rest, eset) = match parse_eset(buffer) {
        Ok(result) => result,
        Err(e) => {
            return Err(anyhow!("Failed to parse Editor Charset: \n{}", e));
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

pub fn process_ps24(buffer: &[u8], _opt: &Options) -> anyhow::Result<()> {
    let (rest, pset) = match parse_ps24(buffer) {
        Ok(result) => result,
        Err(e) => {
            return Err(anyhow!("Failed to parse Editor Charset: \n{}", e));
        }
    };

    if !rest.is_empty() {
        println!("Unconsumed input: {:#?}", Buf(rest));
    }

    fn print_border(w: u8) {
        print!("+");
        for _ in 0..w {
            print!("--------");
        }
        println!("+");
    }

    for glyph in pset.chars {
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

    Ok(())
}

pub fn process_ls30(buffer: &[u8], _opt: &Options) -> anyhow::Result<()> {
    let (rest, lset) = match parse_ls30(buffer) {
        Ok(result) => result,
        Err(e) => {
            return Err(anyhow!("Failed to parse Editor Charset: \n{}", e));
        }
    };

    if !rest.is_empty() {
        println!("Unconsumed input: {:#?}", Buf(rest));
    }

    fn print_border(w: u8) {
        print!("+");
        for _ in 0..w {
            print!("--------");
        }
        println!("+");
    }

    for glyph in lset.chars {
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

    Ok(())
}
