use core::fmt::Write;

pub struct Buf<const N: usize> {
    data: [u8; N],
    pub len: usize,
}

impl<const N: usize> Buf<N> {
    pub const fn new() -> Self {
        Self {
            data: [0; N],
            len: 0,
        }
    }

    pub fn push(&mut self, bytes: &[u8]) {
        let end = self.len + bytes.len();
        self.data[self.len..end].copy_from_slice(bytes);
        self.len = end;
    }

    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.data[..self.len]) }
    }

    fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }

    pub fn as_signed_slice(&self) -> &[i8] {
        unsafe { core::slice::from_raw_parts(self.as_slice().as_ptr().cast::<i8>(), self.len) }
    }

    pub const fn clear(&mut self) {
        self.len = 0;
    }
}

impl<const N: usize> Write for Buf<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        if self.len + bytes.len() > N {
            return Err(core::fmt::Error);
        }
        self.push(bytes);
        Ok(())
    }
}
