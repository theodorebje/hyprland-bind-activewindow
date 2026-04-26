use crate::SUN_PATH_SIZE;
use core::{
    ffi::{c_int, c_void},
    mem::MaybeUninit,
};
use libc::{SOCK_STREAM, sockaddr_un};

#[allow(clippy::cast_possible_truncation)] // AF_UNIX is 1, far below u16::MAX
const AF_UNIX: libc::sa_family_t = libc::AF_UNIX as libc::sa_family_t;

pub struct UnixStream {
    fd: c_int,
}

#[derive(Debug, Clone, Copy)]
pub struct SocketPath(pub [i8; SUN_PATH_SIZE]);

impl SocketPath {
    pub const fn new() -> Self {
        Self([0; 108])
    }
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
    pub fn write(&self, buf: &[i8]) -> Result<usize, i32> {
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
    pub fn connect(path: SocketPath) -> Result<Self, i32> {
        // Create socket
        let fd = unsafe { libc::socket(i32::from(AF_UNIX), SOCK_STREAM, 0) };
        if fd < 0 {
            return Err(unsafe { *libc::__errno_location() });
        }

        // Build sockaddr_un
        let mut addr: sockaddr_un = unsafe { MaybeUninit::zeroed().assume_init() };
        addr.sun_family = AF_UNIX;

        // Copy the entire SocketPath array (already null‑terminated, user must ensure)
        addr.sun_path = path.0;

        // Find the length of the null‑terminated string inside sun_path
        // SAFETY: The user guarantees that `path.0` contains a valid null‑terminated C string
        //         of length at most 107 (leaving room for the null terminator).
        let path_len = unsafe { libc::strlen(addr.sun_path.as_ptr()) };
        // Ensure the path is not longer than the buffer (should never happen with correct input)
        if path_len >= addr.sun_path.len() {
            unsafe { libc::close(fd) };
            return Err(libc::EINVAL); // Or any appropriate error
        }

        let addr_ptr = (&raw const addr).cast::<libc::sockaddr>();
        let addr_len =
            libc::socklen_t::try_from(core::mem::offset_of!(sockaddr_un, sun_path) + path_len + 1)
                .unwrap();

        let ret = unsafe { libc::connect(fd, addr_ptr, addr_len) };
        if ret < 0 {
            let err = unsafe { *libc::__errno_location() };
            unsafe { libc::close(fd) };
            return Err(err);
        }

        Ok(Self::from_raw_fd(fd))
    }

    pub fn write_all(&self, mut buf: &[i8]) -> Result<(), i32> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => return Err(libc::EPIPE), // EOF unexpectedly
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

impl Drop for UnixStream {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}
