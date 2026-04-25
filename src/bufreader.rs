use crate::unixstream::UnixStream;
use alloc::vec;
use alloc::vec::Vec;

pub struct BufReader<R> {
    inner: R,
    buf: Vec<u8>, // internal buffer
    pos: usize,   // next byte to read from buf
    cap: usize,   // number of valid bytes in buf
}

impl<R> BufReader<R> {
    /// Create a new `BufReader` with a default buffer size (e.g. 4 KiB).
    pub fn new(inner: R) -> Self {
        Self::with_capacity(2048, inner)
    }

    pub fn with_capacity(capacity: usize, inner: R) -> Self {
        Self {
            inner,
            buf: vec![0u8; capacity],
            pos: 0,
            cap: 0,
        }
    }

    /// Returns a reference to the filled part of the buffer.
    pub fn fill_buf(&mut self) -> Result<&[u8], i32>
    where
        R: Read, // we'll define a simple Read trait
    {
        if self.pos >= self.cap {
            // buffer is empty, refill
            self.cap = self.inner.read(&mut self.buf)?;
            self.pos = 0;
        }
        Ok(&self.buf[self.pos..self.cap])
    }

    /// Consume `amt` bytes from the buffer.
    pub const fn consume(&mut self, amt: usize) {
        self.pos += amt;
        if self.pos >= self.cap {
            self.pos = 0;
            self.cap = 0;
        }
    }

    /// Read a line (terminated by `b'\n'`) into a `String`.
    pub fn read_line(&mut self, output: &mut alloc::string::String) -> Result<usize, i32>
    where
        R: Read,
    {
        output.clear();
        let mut total = 0;
        loop {
            let (consume_len, line_bytes) = {
                let available = self.fill_buf()?;
                if available.is_empty() {
                    break;
                }
                available.iter().position(|&b| b == b'\n').map_or_else(
                    || {
                        let consume = available.len();
                        let line_bytes = available.to_vec(); // copy whole buffer
                        (consume, line_bytes)
                    },
                    |pos| {
                        let consume = pos + 1; // include the newline
                        let line_bytes = available[..consume].to_vec(); // copy
                        (consume, line_bytes)
                    },
                )
            };
            // Now the borrow from fill_buf is gone, we can call consume
            output.push_str(core::str::from_utf8(&line_bytes).map_err(|_| libc::EILSEQ)?);
            self.consume(consume_len);
            total += consume_len;
            if line_bytes.last() == Some(&b'\n') {
                break;
            }
        }
        Ok(total)
    }
}

/// A very simple `Read` trait that only needs `read`.
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, i32>;
}

impl Read for UnixStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, i32> {
        Self::read(self, buf) // reuse the earlier method
    }
}
