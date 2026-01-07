use crate::{bits::BitIter, terminals, Color};

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
        if let Some(code) = terminals::fax_decode_h(bit_iter, terminal) {
            let cu = code as usize;
            a0 += cu;
            output.reserve(cu);
            output.extend(std::iter::repeat_n(color, cu));
        }
        color.invert();
    }
    output
}
