use std::{io, iter::Peekable};

use crate::{ascii_art::BorderDrawing, ASCII};

/// A decoded bi-level image
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

    /// Print the image to the console / stdout
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

    /// Write a PBM file from the decoded image
    ///
    /// ## Parameters
    ///
    /// - `dbl` - write every line twice, because "Standard" fax is 200 dpi horizontal and 100 dpi vertical
    pub fn write_pbm<W: io::Write>(
        &self,
        writer: &mut W,
        dbl: bool,
        invert: bool,
    ) -> io::Result<()> {
        let mut height = self.complete.len().div_ceil(self.width);
        if dbl {
            height <<= 1;
        }
        writeln!(writer, "P1 {} {}", self.width, height)?;
        for row in RepeatIter::new(self.complete.chunks_exact(self.width), 2) {
            for bit in row {
                // PBM: 1 is black, 0 is white
                let v = if *bit ^ invert { 1 } else { 0 };
                write!(writer, "{:b}", v)?;
            }
            writeln!(writer)?;
        }
        Ok(())
    }
}

struct RepeatIter<I: Iterator> {
    inner: Peekable<I>,
    rem: usize,
    count: usize,
}

impl<I: Iterator> RepeatIter<I>
where
    I::Item: Copy,
{
    fn new(inner: I, count: usize) -> Self {
        assert!(count > 0);
        Self {
            count,
            rem: count - 1,
            inner: inner.peekable(),
        }
    }
}

impl<T: Copy, I: Iterator<Item = T>> Iterator for RepeatIter<I> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rem == 0 {
            self.rem = self.count - 1;
            self.inner.next()
        } else {
            self.rem -= 1;
            self.inner.peek().copied()
        }
    }
}
