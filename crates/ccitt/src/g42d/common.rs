#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl From<bool> for Color {
    fn from(b: bool) -> Color {
        if b {
            Color::Black
        } else {
            Color::White
        }
    }
}

impl Color {
    pub fn _print_row(row: &[Color]) {
        print!("|");
        for pixel in row {
            match pixel {
                Color::Black => {
                    print!(" ");
                }
                Color::White => {
                    print!("#");
                }
            }
        }
        println!("|");
    }

    pub fn _print_vec(vec: &[Color], width: usize) {
        print!("+");
        for _ in 0..width {
            print!("-");
        }
        println!("+");
        for row in vec.chunks(width) {
            Self::_print_row(row);
        }
        print!("+");
        for _ in 0..width {
            print!("-");
        }
        println!("+");
    }
}

impl Color {
    pub fn invert(&mut self) {
        match self {
            Color::White => {
                *self = Color::Black;
            }
            Color::Black => {
                *self = Color::White;
            }
        }
    }
}
