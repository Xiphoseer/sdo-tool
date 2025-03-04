#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[rustfmt::skip]
struct State(u8);

impl State {
    fn as_usize(&self) -> usize {
        self.0 as usize
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
            state: State(7),
            curr: 0,
        }
    }

    /// Creates a new instance with the given capacity of bits
    pub fn _with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity / 8 + (capacity % 8).min(1)),
            state: State(7),
            curr: 0,
        }
    }

    pub fn write_bit(&mut self, bit: bool) {
        let avail = self.avail();
        let val = if bit { 1 } else { 0 };
        if avail < 8 {
            self.curr <<= 1;
            self.curr |= val;
            self.state = State((avail - 2) as u8);
        } else {
            // at this point, the writer is starting the next byte
            self.curr = val;
            self.state = State(6u8);
        }
    }

    fn avail(&self) -> usize {
        self.state.as_usize() + 1
    }

    /// Write {off} bits of {val}
    pub fn write_bits(&mut self, val: usize, mut todo: usize) {
        let avail = self.avail();
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
        self.state = State((7 - todo) as u8);
    }

    /// flush the output buffer
    pub fn flush(&mut self) {
        let offset = self.state.as_usize() + 1;
        if offset < 8 {
            self.curr <<= offset;
            self.buffer.push(self.curr);
            self.curr = 0;
            self.state = State(7);
        }
    }

    /// Flush and return the buffer
    pub fn done(mut self) -> Vec<u8> {
        self.flush();
        self.buffer
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_bit_writer() {
        let mut bit_writer = super::BitWriter::new();
        bit_writer.write_bits(0b111, 3);
        bit_writer.write_bits(0, 4);
        bit_writer.write_bits(0b11, 2);
        bit_writer.write_bits(0, 7);
        bit_writer.write_bits(0b10101, 5);
        bit_writer.flush();
        let vec = bit_writer.done();
        assert_eq!(vec![0b11100001, 0b10000000, 0b10101000], vec);
    }
}
