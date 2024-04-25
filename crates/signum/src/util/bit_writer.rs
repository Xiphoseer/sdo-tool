#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[rustfmt::skip]
#[repr(u8)]
#[allow(dead_code)]
enum State { #[default] S0, S1, S2, S3, S4, S5, S6, S7 }

impl State {
    /*#[rustfmt::skip]
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
    }*/

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
    /*pub fn write(&mut self, b: bool) {
        self.curr <<= 1;
        if b {
            self.curr |= 1;
        }
        if self.state.tick() {
            self.buffer.push(self.curr);
            self.curr = 0;
        }
    }*/

    /// Write {off} bits of {val}
    pub fn write_bits(&mut self, val: usize, mut todo: usize) {
        let avail = self.state.as_usize() + 1;
        if avail < 8 {
            if todo < avail {
                self.curr <<= todo;
                let mask = (1 << todo) - 1;
                self.curr |= (val & mask) as u8;
                self.state = unsafe {
                    // This is safe, because `todo < avail < 8`
                    // and state + 1 = avail, so todo <= state
                    std::mem::transmute((avail - 1 - todo) as u8)
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
            std::mem::transmute((7 - todo) as u8)
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
