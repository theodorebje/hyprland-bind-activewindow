use crate::{
    SUN_PATH_SIZE,
    libasm::{EPIPE, close, connect_to_socket, read, write_signed},
};
use core::ffi::c_int;

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
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, i32> {
        read(self.fd, buf)
    }

    fn write(&self, buf: &[i8]) -> Result<usize, i32> {
        write_signed(self.fd, buf)
    }

    pub fn connect(path: SocketPath) -> Result<Self, i32> {
        Ok(Self {
            fd: connect_to_socket(path)?,
        })
    }

    pub fn write_all(&self, mut buf: &[i8]) -> Result<(), i32> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => return Err(EPIPE),
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

impl Drop for UnixStream {
    fn drop(&mut self) {
        close(self.fd).expect("failed to close the Unix socket file descriptor");
    }
}
