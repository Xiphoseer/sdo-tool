use crate::{font::dvips::parse_fa, util::data::BIT_STRING};
use anyhow::anyhow;


pub fn process_ps_font(buffer: &[u8]) -> anyhow::Result<()> {
    let fa = match parse_fa(&buffer) {
        Ok((_, fa)) => fa,
        Err(nom::Err::Failure((rest, e))) => {
            return Err(anyhow!("Parse failure: {:?}\n{}", e, std::str::from_utf8(rest).unwrap()));
        }
        Err(nom::Err::Error((rest, e))) => {
            return Err(anyhow!("Parse error: {:?}\n{}", e, std::str::from_utf8(rest).unwrap()));
        }
        Err(nom::Err::Incomplete(_)) => {
            return Err(anyhow!("Incomplete"));
        }
    };
    println!("Font: {} of {}", fa.len, fa.max);
    for ch in fa.chars {
        let bytes = &ch.stream.inner;
        let (data, header) = bytes.split_at(bytes.len() - 5);

        println!("{:?}", header);
        let w = data.len() / header[1] as usize;
        let width = header[0];
        let rest = width % 8;
        
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
        //println!("{:#?}", Buf(data));
    }
    Ok(())
}