use crate::{bits::BitWriter, Color};

/// This struct can represents a scanline
pub trait ColorLine {
    /// Get the color at index i
    fn color_at(&self, i: usize) -> Color;
    /// Set the color at index i
    fn set_color(&mut self, i: usize, color: Color);
}

/// This struct can store a bitmap
pub trait Store {
    /// The type of scanline
    type Row: ColorLine;

    /// Create a new struct
    fn new() -> Self;
    /// Create the next row
    fn new_row(width: usize) -> Self::Row;
    /// Add a row to the bitmap
    fn extend(&mut self, row: &Self::Row);
}

impl Store for BitWriter {
    type Row = Vec<Color>;

    fn new() -> Self {
        BitWriter::new()
    }

    fn new_row(width: usize) -> Self::Row {
        vec![Color::White; width]
    }

    fn extend(&mut self, row: &Self::Row) {
        for color in row {
            self.write(*color == Color::Black);
        }
        // TODO: flush here?
    }
}

impl ColorLine for Vec<Color> {
    fn color_at(&self, i: usize) -> Color {
        if i == 0 {
            Color::White
        } else {
            self[i - 1]
        }
    }

    fn set_color(&mut self, i: usize, color: Color) {
        if i == 0 {
            //println!("WARN: trying to assign color to index 0")
        } else {
            self[i - 1] = color;
        }
    }
}

impl Store for Vec<Color> {
    type Row = Vec<Color>;

    fn new() -> Self {
        vec![]
    }

    fn new_row(width: usize) -> Self::Row {
        vec![Color::White; width]
    }

    fn extend(&mut self, row: &Self::Row) {
        self.extend_from_slice(row);
    }
}
