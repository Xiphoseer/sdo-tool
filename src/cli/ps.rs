use std::io::{self, Write};

use crate::ps::PSWriter;
use crate::{
    font::dvips::parse_char_header, font::dvips::parse_dvips_bitmap_font, font::dvips::CacheDevice,
    font::dvips::CharHeader, font::printer::parse_ls30, font::printer::PSet,
    util::data::BIT_STRING,
};
use anyhow::anyhow;

pub fn write_ls30_ps_bitmap(
    key: &str,
    pw: &mut PSWriter<impl Write>,
    font: &PSet,
) -> io::Result<()> {
    pw.lit(key)?;
    let count = font.chars.iter().filter(|c| c.width > 0).count();
    pw.write_usize(count)?;
    pw.write_usize(128)?;
    pw.name("df")?;

    let mut cc = 0;
    for (i, chr) in font.chars.iter().enumerate() {
        if chr.width > 0 {
            let char_header = CharHeader::from_signum(&chr);
            let head_iter = char_header.iter();
            let iter = chr.bitmap.iter().copied().chain(head_iter);
            pw.write_stream(iter)?;

            if cc == i {
                pw.name("I")?;
            } else {
                pw.write_usize(i)?;
                pw.name("D")?;
            }
            cc = i + 1;
        }
    }
    pw.name("E")?;
    Ok(())
}

pub fn convert_ls30(buffer: &[u8]) -> anyhow::Result<()> {
    let font = match parse_ls30(&buffer) {
        Ok((_, fa)) => fa,
        Err(nom::Err::Failure((rest, e))) => {
            return Err(anyhow!(
                "Parse failure: {:?}\n{}",
                e,
                std::str::from_utf8(rest).unwrap()
            ));
        }
        Err(nom::Err::Error((rest, e))) => {
            return Err(anyhow!(
                "Parse error: {:?}\n{}",
                e,
                std::str::from_utf8(rest).unwrap()
            ));
        }
        Err(nom::Err::Incomplete(_)) => {
            return Err(anyhow!("Incomplete"));
        }
    };

    let mut writer = PSWriter::new();
    write_ls30_ps_bitmap("Fa", &mut writer, &font)?;

    Ok(())
}

pub fn process_ps_font(buffer: &[u8]) -> anyhow::Result<()> {
    let fa = match parse_dvips_bitmap_font(&buffer) {
        Ok((_, fa)) => fa,
        Err(nom::Err::Failure((rest, e))) => {
            return Err(anyhow!(
                "Parse failure: {:?}\n{}",
                e,
                std::str::from_utf8(rest).unwrap()
            ));
        }
        Err(nom::Err::Error((rest, e))) => {
            return Err(anyhow!(
                "Parse error: {:?}\n{}",
                e,
                std::str::from_utf8(rest).unwrap()
            ));
        }
        Err(nom::Err::Incomplete(_)) => {
            return Err(anyhow!("Incomplete"));
        }
    };
    println!("Font: {} of {}", fa.len, fa.max);
    for ch in fa.chars {
        let bytes = &ch.stream.inner;
        let (data, head_bytes) = bytes.split_at(bytes.len() - 5);
        let header = parse_char_header(head_bytes).unwrap().1;
        println!("{:?}", header);
        let frame = CacheDevice::from(header);
        println!("{:?}", frame);

        let w = data.len() / header.height as usize;
        let rest = header.width % 8;

        let border = || {
            print!("+");
            for _ in 0..(w - 1) {
                print!("--------");
            }
            if rest == 0 {
                print!("--------");
            } else {
                print!("{}", &"--------"[..(rest as usize)]);
            }
            println!("+");
        };
        border();
        for row in data.chunks_exact(w) {
            print!("|");
            for i in 0..(w - 1) {
                print!("{}", &BIT_STRING[row[i] as usize]);
            }
            if rest == 0 {
                print!("{}", &BIT_STRING[row[w - 1] as usize]);
            } else {
                print!("{}", &BIT_STRING[row[w - 1] as usize][..(rest as usize)]);
            }
            println!("|");
        }
        border();
    }
    Ok(())
}
