//! Common structs and enums

/// Black or White Color
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Color {
    /// No-Ink
    White = 0xFF,
    /// Ink
    Black = 0x00,
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

impl From<Color> for bool {
    fn from(c: Color) -> bool {
        c == Color::Black
    }
}

impl std::ops::Not for Color {
    type Output = Color;

    fn not(self) -> Self::Output {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
}

#[cfg(feature = "debug")]
impl Color {
    /// Print a monochrome scanline
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

    /// Print a monochrome bitmap
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
    /// Invert a color
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
