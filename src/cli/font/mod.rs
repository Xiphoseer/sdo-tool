use crate::cli::opt::{Format, Options};
use ccitt_t4_t6::g42d::encode::Encoder;
use color_eyre::eyre::{self, eyre};
use image::ImageFormat;
use sdo::{
    font::{
        editor::parse_eset,
        printer::{parse_ls30, parse_ps24},
    },
    print::Page,
    ps::PSWriter,
    util::{data::BIT_STRING, Buf},
};
use std::path::PathBuf;

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

pub fn process_ps24(buffer: &[u8], _opt: &Options) -> eyre::Result<()> {
    let (rest, pset) = match parse_ps24(buffer) {
        Ok(result) => result,
        Err(e) => {
            return Err(eyre!("Failed to parse Editor Charset: \n{}", e));
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

pub fn process_ls30(buffer: &[u8], opt: &Options) -> eyre::Result<()> {
    let (rest, lset) = match parse_ls30(buffer) {
        Ok(result) => result,
        Err(e) => {
            return Err(eyre!("Failed to parse Editor Charset: \n{}", e));
        }
    };

    if !rest.is_empty() {
        println!("Unconsumed input: {:#?}", Buf(rest));
    }

    if opt.format == Format::DVIPSBitmapFont {
        let mut writer = PSWriter::new();
        write_ls30_ps_bitmap("Fa", "FONT", &mut writer, &lset, None)?;
        return Ok(());
    } else if opt.format == Format::CCITTT6 {
        std::fs::create_dir_all(&opt.out)?;
        for (cval, chr) in lset.chars.iter().enumerate() {
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
                let path = opt.out.join(file);
                std::fs::write(path, contents)?;
            }
        }
        return Ok(());
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
