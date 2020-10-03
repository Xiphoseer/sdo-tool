use std::io::{self, Write};

use color_eyre::eyre::{self, eyre};
use nom::Finish;
use sdo::{
    font::dvips::parse_char_header, font::dvips::parse_dvips_bitmap_font, font::dvips::CacheDevice,
    font::dvips::CharHeader, font::printer::PSet, font::UseTable, nom, ps::PSWriter,
    util::data::BIT_STRING,
};

pub fn write_ls30_ps_bitmap(
    key: &str,
    name: &str,
    pw: &mut PSWriter<impl Write>,
    font: &PSet,
    use_table: Option<&UseTable>,
) -> io::Result<()> {
    pw.lit(key)?;
    let count = font.chars.iter().filter(|c| c.width > 0).count();
    pw.bytes(name.as_bytes())?;
    pw.write_usize(count)?;
    pw.write_usize(128)?;
    pw.name("df")?;

    let mut cc = 0;
    for (i, chr) in font.chars.iter().enumerate() {
        let used = use_table.map(|arr| arr.chars[i as usize] > 0);
        if chr.width > 0 && used != Some(false) {
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
        } else if used == Some(true) {
            println!(
                "Warning: Font `{}`: Non-renderable character is used #{}",
                key, i
            );
        }
    }
    pw.name("E")?;
    Ok(())
}

pub fn process_ps_font(buffer: &[u8]) -> eyre::Result<()> {
    let (_, fa) = parse_dvips_bitmap_font(&buffer)
        .finish()
        .map_err(|e| eyre!("Faile to parse DVIPSBitmapFont: {:?}", e))?;
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
