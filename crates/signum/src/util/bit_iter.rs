//! # A naive bit iterator

use std::{
    iter::{Copied, FlatMap},
    slice::Iter,
};

/// Iterator over the bits in a byte
pub struct ByteBits {
    size: u8,
    bits: u8,
}

impl ByteBits {
    /// Create a new bit iterator from a byte
    pub fn new(bits: u8) -> Self {
        Self {
            size: u8::BITS as u8,
            bits,
        }
    }
}

impl Iterator for ByteBits {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.size > 0 {
            self.size -= 1;
            let (bits, carry) = self.bits.overflowing_mul(2);
            self.bits = bits;
            Some(carry)
        } else {
            None
        }
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.size.into()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.size.into();
        (size, Some(size))
    }
}

type ByteIter<'a> = Copied<Iter<'a, u8>>;

/// A bit iterator
pub struct BitIter<'a> {
    inner: FlatMap<ByteIter<'a>, ByteBits, fn(u8) -> ByteBits>,
}

impl<'a> BitIter<'a> {
    /// Create a new bit iter from a byte slice
    pub fn new(bytes: &'a [u8]) -> BitIter<'a> {
        Self {
            inner: bytes.iter().copied().flat_map(ByteBits::new),
        }
    }
}

impl Iterator for BitIter<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.inner.count() // FIXME: optimize
    }
}

#[cfg(test)]
mod tests {
    use crate::util::BitIter;

    use super::ByteBits;

    #[test]
    fn byte_bits() {
        assert_eq!(
            vec![false, false, true, false, false, false, false, false],
            ByteBits::new(0b0010_0000).collect::<Vec<_>>()
        );
    }

    #[test]
    fn bit_iter() {
        assert_eq!(
            vec![
                false, false, true, false, false, false, false, false, //
                true, true, false, false, true, true, false, true
            ],
            BitIter::new(&[0b0010_0000, 0b1100_1101]).collect::<Vec<_>>()
        );
    }
}
