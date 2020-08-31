//! # Image formats
pub mod imc;
pub mod native;

pub struct BitIter<'a> {
    state: u8,
    buffer: u8,
    inner: std::slice::Iter<'a, u8>,
}

impl<'a> BitIter<'a> {
    pub fn new(bytes: &'a [u8]) -> BitIter<'a> {
        BitIter {
            state: 0,
            buffer: 0,
            inner: bytes.iter(),
        }
    }
}

impl Iterator for BitIter<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == 0 {
            self.state = 7;
            if let Some(value) = self.inner.next() {
                self.buffer = *value;
            } else {
                return None;
            }
        } else {
            self.state -= 1;
        }
        let (next_buffer, carry) = self.buffer.overflowing_mul(2);
        self.buffer = next_buffer;
        Some(carry)
    }
}
