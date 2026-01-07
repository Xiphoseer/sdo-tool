use crate::{
    bits::BitIter,
    terminals::{self, Terminal},
    Color,
};

/// Decode a line of 1-d encoded bi-level image data
pub fn decode_1d_line(bit_iter: &mut BitIter<'_>, width: usize) -> Vec<Color> {
    let mut color = Color::White;
    let mut a0 = 0;
    let mut output = Vec::new();

    // TODO: decode EOL at start of line
    while a0 < width {
        let terminal = match color {
            Color::White => terminals::white_terminal,
            Color::Black => terminals::black_terminal,
        };
        match terminals::fax_decode_h(bit_iter, terminal) {
            Some(Terminal::Sum(code)) => {
                let cu = code as usize;
                a0 += cu;
                output.reserve(cu);
                output.extend(std::iter::repeat_n(color, cu));
                color.invert();
            }
            Some(Terminal::EOL) => {
                if !output.is_empty() {
                    break;
                }
            }
            Some(Terminal::Code10) => {
                // Extension?
                let a = bit_iter.next().unwrap();
                let b = bit_iter.next().unwrap();
                let c = bit_iter.next().unwrap();
                panic!("Extension? at {} ({},{},{})", output.len(), a, b, c);
            }
            Some(term) => panic!("{:?}", term),
            None => {
                if !output.is_empty() {
                    println!("WARN: EOF mid scanline");
                }
                break;
            }
        }
    }
    output
}
