//! Generic utilities

use std::io::{self, Write};

/// Source: <https://stackoverflow.com/questions/42187591/>
pub struct ByteCounter<W> {
    inner: W,
    count: usize,
}

impl<W> ByteCounter<W>
where
    W: Write,
{
    /// Create a new byte counter
    pub fn new(inner: W) -> Self {
        ByteCounter { inner, count: 0 }
    }

    /// Return the inner writer
    pub fn into_inner(self) -> W {
        self.inner
    }

    /// Get the number of bytes written
    pub fn bytes_written(&self) -> usize {
        self.count
    }
}

impl<W> Write for ByteCounter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let res = self.inner.write(buf);
        if let Ok(size) = res {
            self.count += size
        }
        res
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

pub(crate) struct NextId {
    obj_id: u64,
}

impl NextId {
    pub(crate) fn new(start: u64) -> Self {
        Self { obj_id: start }
    }

    pub(crate) fn next(&mut self) -> u64 {
        let next = self.obj_id;
        self.obj_id += 1;
        next
    }
}
