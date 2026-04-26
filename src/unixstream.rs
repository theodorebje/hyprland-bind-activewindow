use crate::{
    SUN_PATH_SIZE,
    libasm::{EPIPE, close, connect_to_socket, read, write_signed},
};
use core::
    ffi::c_int
;

pub struct UnixStream {
    fd: c_int,
}

#[derive(Debug, Clone, Copy)]
pub struct SocketPath(pub [i8; SUN_PATH_SIZE]);

impl SocketPath {
    pub const fn new() -> Self {
        Self([0; SUN_PATH_SIZE])
    }
}

impl UnixStream {
    /// Wrap an existing file descriptor (e.g. from `socket` or `accept`).
    /// The fd must be valid and will be closed on Drop.
    pub const fn from_raw_fd(fd: c_int) -> Self {
        Self { fd }
    }

    /// Read up to `buf.len()` bytes into `buf`. Returns number of bytes read,
    /// or a negative errno on failure.
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, i32> {
        read(self.fd, buf)
    }

    /// Write `buf` to the stream. Returns number of bytes written or error.
    pub fn write(&self, buf: &[i8]) -> Result<usize, i32> {
        write_signed(self.fd, buf)
    }

    /// Connect to a Unix domain socket at the given path.
    ///
    /// # Errors
    /// Returns `Err(errno)` if socket creation or connection fails.
    pub fn connect(path: SocketPath) -> Result<Self, i32> {
        Ok(Self::from_raw_fd(connect_to_socket(path)?))
    }

    pub fn write_all(&self, mut buf: &[i8]) -> Result<(), i32> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => return Err(EPIPE), // EOF unexpectedly
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

impl Drop for UnixStream {
    fn drop(&mut self) {
        close(self.fd).unwrap();
    }
}
