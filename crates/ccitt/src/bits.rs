//! # Bit Iterator and Writer

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[rustfmt::skip]
#[repr(u8)]
enum State { #[default] S0, S1, S2, S3, S4, S5, S6, S7 }

impl State {
    #[rustfmt::skip]
    fn tick(&mut self) -> bool {
        use State::*;
        match self {
            S0 => { *self = S7; true}
            S1 => { *self = S0; false}
            S2 => { *self = S1; false}
            S3 => { *self = S2; false}
            S4 => { *self = S3; false}
            S5 => { *self = S4; false}
            S6 => { *self = S5; false}
            S7 => { *self = S6; false}
        }
    }

    fn as_usize(&self) -> usize {
        match self {
            Self::S0 => 0,
            Self::S1 => 1,
            Self::S2 => 2,
            Self::S3 => 3,
            Self::S4 => 4,
            Self::S5 => 5,
            Self::S6 => 6,
            Self::S7 => 7,
        }
    }
}

/// A bitwise writer
#[derive(Debug)]
pub struct BitWriter {
    buffer: Vec<u8>,
    state: State,
    curr: u8,
}

impl Default for BitWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl BitWriter {
    /// Creates a new instance
    pub fn new() -> Self {
        Self {
            buffer: vec![],
            state: State::S7,
            curr: 0,
        }
    }

    /// Creates a new instance with the given capacity of bits
    pub fn _with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity / 8 + (capacity % 8).min(1)),
            state: State::S7,
            curr: 0,
        }
    }

    /// Write a single bit
    pub fn write(&mut self, b: bool) {
        self.curr <<= 1;
        if b {
            self.curr |= 1;
        }
        if self.state.tick() {
            self.buffer.push(self.curr);
            self.curr = 0;
        }
    }

    /// Write {off} bits of {val}
    pub fn write_bits(&mut self, val: usize, off: u8) {
        let avail = self.state.as_usize() + 1;
        let mut todo = off as usize;
        if avail < 8 {
            if todo < avail {
                self.curr <<= todo;
                let mask = (1 << todo) - 1;
                self.curr |= (val & mask) as u8;
                self.state = unsafe {
                    // This is safe, because `todo < avail < 8`
                    // and state + 1 = avail, so todo <= state
                    std::mem::transmute::<u8, State>((avail - 1 - todo) as u8)
                };
                return;
            } else {
                let rest = todo - avail;
                let prefix = (val >> rest) as u8;
                self.buffer.push(self.curr << avail | prefix);
                todo = rest;
            }
        }
        // at this point, the writer is starting the next byte
        while todo >= 8 {
            let rest = todo - 8;
            let middle = ((val >> rest) & 0xFF) as u8;
            self.buffer.push(middle);
            todo = rest;
            self.curr = 0;
        }
        // at this point, the writer is starting the next byte
        let mask = (1 << todo) - 1;
        self.curr = (mask & val) as u8;
        self.state = unsafe {
            // This is safe, because todo < 8 as per the loop above
            std::mem::transmute::<u8, State>((7 - todo) as u8)
        };
    }

    /// flush the output buffer
    pub fn flush(&mut self) {
        let offset = self.state.as_usize() + 1;
        if offset < 8 {
            self.curr <<= offset;
            self.buffer.push(self.curr);
            self.curr = 0;
            self.state = State::S7;
        }
    }

    /// Flush and return the buffer
    pub fn done(mut self) -> Vec<u8> {
        self.flush();
        self.buffer
    }
}

/// Order of writing/reading bits to/from a byte (see TIFF spec)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FillOrder {
    /// A byte is iterated from most- to least-significant bit
    #[default]
    MsbToLsb = 1,
    /// A byte is iterated from lest- to most-significant bit
    LsbToMsb = 2,
}

impl FillOrder {
    fn next(&self, buffer: u8) -> (u8, bool) {
        match self {
            FillOrder::MsbToLsb => buffer.overflowing_mul(2),
            FillOrder::LsbToMsb => (buffer >> 1, buffer & 0b1 > 0),
        }
    }
}

/// Read bits from a slice
#[derive(Debug, Clone)]
pub struct BitIter<'a> {
    state: State,
    fill_order: FillOrder,
    buffer: u8,
    inner: std::slice::Iter<'a, u8>,
}

impl<'a> BitIter<'a> {
    /// Creates a new instance
    pub fn new(bytes: &'a [u8]) -> BitIter<'a> {
        BitIter {
            state: State::default(),
            fill_order: FillOrder::MsbToLsb,
            buffer: 0,
            inner: bytes.iter(),
        }
    }

    /// Update the fill order. This should be done before
    /// any call to next, otherwise the resulting stream
    /// may be corrupt, but it's not unsound.
    pub fn set_fill_order(&mut self, fill_order: FillOrder) {
        self.fill_order = fill_order;
    }

    /// Get the next two bits
    pub fn next_2(&mut self) -> Option<(bool, bool)> {
        let a = self.next()?;
        let b = self.next()?;
        Some((a, b))
    }

    fn cli_image_inner(&mut self, width: usize) -> bool {
        for _ in 0..width {
            match self.next() {
                Some(true) => {
                    print!(" ");
                }
                Some(false) => {
                    print!("#");
                }
                None => {
                    return false;
                }
            }
        }
        true
    }

    /// Draw the image to the console
    pub fn cli_image(mut self, width: usize) {
        print!("+");
        for _ in 0..width {
            print!("-");
        }
        println!("+");
        loop {
            print!("|");
            let cont = self.cli_image_inner(width);
            println!("|");
            if !cont {
                break;
            }
        }
        print!("+");
        for _ in 0..width {
            print!("-");
        }
        println!("+");
    }
}

impl Iterator for BitIter<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state.tick() {
            if let Some(value) = self.inner.next() {
                self.buffer = *value;
            } else {
                self.state = State::S0;
                return None;
            }
        }
        let (next_buffer, bit) = self.fill_order.next(self.buffer);
        self.buffer = next_buffer;
        Some(bit)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.inner.size_hint().0 * 8 + self.state.as_usize();
        (size, Some(size))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.inner.count() * 8 + self.state.as_usize()
    }
}

#[cfg(test)]
mod tests {
    use super::{BitWriter, FillOrder, State};

    #[test]
    fn test_fill_order_msb_to_lsb() {
        let msbf = FillOrder::MsbToLsb;
        assert_eq!(msbf.next(0b10000000), (0b00000000, true));
        assert_eq!(msbf.next(0b01000000), (0b10000000, false));
        assert_eq!(msbf.next(0b10100000), (0b01000000, true));
    }

    #[test]
    fn test_fill_order_lsb_to_msb() {
        let msbf = FillOrder::LsbToMsb;
        assert_eq!(msbf.next(0b00000001), (0b00000000, true));
        assert_eq!(msbf.next(0b00000010), (0b00000001, false));
        assert_eq!(msbf.next(0b00000101), (0b00000010, true));
    }

    #[test]
    fn test_bit_writer_write_bits() {
        let mut bw = BitWriter::new();
        bw.write_bits(0b000011110000, 12);
        bw.write_bits(0b1010, 4);

        assert_eq!(&bw.buffer, &[0b00001111, 0b00001010]);

        bw.write_bits(0b111111, 6);
        assert_eq!(bw.curr, 0b111111);
        assert_eq!(bw.state, State::S1);
        assert_eq!(&bw.buffer, &[0b00001111, 0b00001010]);

        bw.write_bits(0b000000, 6);
        assert_eq!(bw.curr, 0b0000);
        assert_eq!(bw.state, State::S3);
        assert_eq!(&bw.buffer, &[0b00001111, 0b00001010, 0b11111100]);

        bw.write_bits(0b1111, 4);
        assert_eq!(
            &bw.buffer,
            &[0b00001111, 0b00001010, 0b11111100, 0b00001111]
        );
        assert_eq!(bw.curr, 0);
        assert_eq!(bw.state, State::S7);
    }

    #[test]
    fn test_bit_writer_write() {
        let mut bw = BitWriter::new();
        bw.write(true);
        bw.write(false);
        bw.write(false);
        bw.write(true);
        bw.write(true);
        bw.write(false);
        bw.write(true);
        bw.write(false);

        assert_eq!(&bw.buffer, &[0b10011010]);

        bw.write(true);
        bw.write(true);
        bw.write(true);
        bw.write(true);
        bw.write(false);
        bw.write(false);
        bw.write(false);
        bw.write(false);

        assert_eq!(&bw.buffer, &[0b10011010, 0b11110000]);

        bw.write(true);
        bw.write(false);
        bw.write(true);
        bw.flush();

        assert_eq!(&bw.buffer, &[0b10011010, 0b11110000, 0b10100000]);

        bw.flush();

        assert_eq!(&bw.buffer, &[0b10011010, 0b11110000, 0b10100000]);
    }
}
