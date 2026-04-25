use alloc::vec::Vec;
use core::{
    ffi::{c_int, c_void},
    mem::{MaybeUninit, offset_of},
};
use libc::{SOCK_STREAM, sockaddr_un};

#[allow(clippy::cast_possible_truncation)] // AF_UNIX is 1, far below u16::MAX
const AF_UNIX: libc::sa_family_t = libc::AF_UNIX as libc::sa_family_t;

pub struct UnixStream {
    fd: c_int,
}

impl UnixStream {
    /// Wrap an existing file descriptor (e.g. from `socket` or `accept`).
    /// The fd must be valid and will be closed on Drop.
    pub const fn from_raw_fd(fd: c_int) -> Self {
        Self { fd }
    }

    /// Read up to `buf.len()` bytes into `buf`. Returns number of bytes read,
    /// or a `libc` error code (negated) on failure.
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, i32> {
        let ret = unsafe { libc::read(self.fd, buf.as_mut_ptr().cast::<c_void>(), buf.len()) };
        if ret < 0 {
            Err(unsafe { *libc::__errno_location() })
        } else {
            Ok(ret.cast_unsigned())
        }
    }

    /// Write `buf` to the stream. Returns number of bytes written or error.
    pub fn write(&self, buf: &[u8]) -> Result<usize, i32> {
        let ret = unsafe { libc::write(self.fd, buf.as_ptr().cast::<c_void>(), buf.len()) };
        if ret < 0 {
            Err(unsafe { *libc::__errno_location() })
        } else {
            Ok(ret.cast_unsigned())
        }
    }

    /// Connect to a Unix domain socket at the given path.
    ///
    /// # Errors
    /// Returns `Err(errno)` if socket creation or connection fails.
    pub fn connect(path: &str) -> Result<Self, i32> {
        // Create socket
        let fd = unsafe { libc::socket(i32::from(AF_UNIX), SOCK_STREAM, 0) };
        if fd < 0 {
            return Err(unsafe { *libc::__errno_location() });
        }

        // Build sockaddr_un
        let mut addr: sockaddr_un = unsafe { MaybeUninit::zeroed().assume_init() };
        addr.sun_family = AF_UNIX;

        // Copy path bytes into sun_path (null-terminated).
        // sun_path is usually 108 bytes long (varies by system). Truncate if too long.
        let bytes = path.as_bytes();
        let len = bytes.len().min(addr.sun_path.len() - 1);
        // Cast the sun_path to a mutable u8 slice
        let sun_path_u8 = unsafe {
            core::slice::from_raw_parts_mut(
                addr.sun_path.as_mut_ptr().cast::<u8>(),
                addr.sun_path.len(),
            )
        };
        sun_path_u8[..len].copy_from_slice(&bytes[..len]);
        sun_path_u8[len] = 0;

        let addr_ptr = (&raw const addr).cast::<libc::sockaddr>();
        let addr_len =
            libc::socklen_t::try_from(offset_of!(sockaddr_un, sun_path) + len + 1).unwrap();

        let ret = unsafe { libc::connect(fd, addr_ptr, addr_len) };
        if ret < 0 {
            let err = unsafe { *libc::__errno_location() };
            // close the socket fd before returning the error
            unsafe { libc::close(fd) };
            return Err(err);
        }

        Ok(Self::from_raw_fd(fd))
    }

    pub fn write_all(&self, mut buf: &[u8]) -> Result<(), i32> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => return Err(libc::EPIPE), // EOF unexpectedly
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> Result<usize, i32> {
        let mut total = 0;
        let mut tmp = [0u8; 2048]; // stack buffer
        loop {
            match self.read(&mut tmp) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    buf.extend_from_slice(&tmp[..n]);
                    total += n;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(total)
    }
}

impl Drop for UnixStream {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}
