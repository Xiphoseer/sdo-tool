use std::io;

use crate::{ascii_art::BorderDrawing, ASCII};

pub struct FaxImage {
    pub(crate) width: usize,
    pub(crate) complete: Vec<bool>,
}

impl FaxImage {
    fn print_border(&self, b: &BorderDrawing) {
        print!("{}", b.left);
        for _ in 0..self.width {
            print!("{}", b.middle);
        }
        println!("{}", b.right);
    }

    pub fn print(&self, invert: bool) {
        let b = ASCII;
        self.print_border(&b.top);
        for row in self.complete.chunks_exact(self.width) {
            print!("{}", b.left);
            for bit in row {
                let c = if *bit ^ invert { b.ink } else { b.no_ink };
                print!("{}", c);
            }
            println!("{}", b.right);
        }
        self.print_border(&b.bottom);
    }

    pub fn write_pbm<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let height = self.complete.len().div_ceil(self.width);
        writeln!(writer, "P1 {} {}", self.width, height)?;
        for row in self.complete.chunks_exact(self.width) {
            for bit in row {
                let v = if *bit { 0 } else { 1 };
                write!(writer, "{:b}", v)?;
            }
            writeln!(writer)?;
        }
        Ok(())
    }
}
