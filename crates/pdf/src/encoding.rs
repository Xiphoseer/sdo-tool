use std::io::{self, Write};

/// Encode the input slice so that it can be decoded with the
/// *Ascii85Decode* filter. Returns the number of written bytes.
pub fn ascii_85_encode<W: Write>(data: &[u8], w: &mut W) -> io::Result<usize> {
    let mut ctr = 0;
    let mut cut = 75;

    let mut chunks_exact = data.chunks_exact(4);
    for group in &mut chunks_exact {
        let buf = u32::from_be_bytes([group[0], group[1], group[2], group[3]]);
        if buf == 0 {
            w.write_all(&[0x7A])?; // `z`
            ctr += 1;
        } else {
            let (c_5, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let (c_4, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let (c_3, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let (c_2, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let c_1 = buf as u8 + 33;
            w.write_all(&[c_1, c_2, c_3, c_4, c_5])?;
            ctr += 5;
        }

        if ctr >= cut {
            w.write_all(&[0x0A])?;
            ctr += 1;
            cut = ctr + 75;
        }
    }
    match *chunks_exact.remainder() {
        [b_1] => {
            let buf = u32::from_be_bytes([b_1, 0, 0, 0]) / (85 * 85 * 85);
            let (c_2, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let c_1 = buf as u8 + 33;
            w.write_all(&[c_1, c_2, 0x7E, 0x3E])?;
            ctr += 4;
        }
        [b_1, b_2] => {
            let buf = u32::from_be_bytes([b_1, b_2, 0, 0]) / (85 * 85);
            let (c_3, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let (c_2, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let c_1 = buf as u8 + 33;
            w.write_all(&[c_1, c_2, c_3, 0x7E, 0x3E])?;
            ctr += 5;
        }
        [b_1, b_2, b_3] => {
            let buf = u32::from_be_bytes([b_1, b_2, b_3, 0]) / 85;
            let (c_4, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let (c_3, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let (c_2, buf) = ((buf % 85) as u8 + 33, buf / 85);
            let c_1 = buf as u8 + 33;
            w.write_all(&[c_1, c_2, c_3, c_4, 0x7E, 0x3E])?;
            ctr += 6;
        }
        _ => {
            w.write_all(&[0x7E, 0x3E])?;
            ctr += 2;
        }
    }

    Ok(ctr)
}
