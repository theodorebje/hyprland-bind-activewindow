use crate::{
    ENV_KEY_SIZE,
    unixstream::{SocketPath, UnixStream},
};
use core::ffi::CStr;

struct StackBuf {
    data: SocketPath,
    len: usize,
}

impl StackBuf {
    const fn new() -> Self {
        Self {
            data: SocketPath::new(),
            len: 0,
        }
    }

    /// Safely push a slice of `u8` bytes (e.g. from `&str`) onto the buffer.
    fn push(&mut self, s: &[u8]) {
        let dest = &mut self.data.0[self.len..self.len + s.len()];
        for (i, &b) in s.iter().enumerate() {
            dest[i] = b.cast_signed();
        }
        self.len += s.len();
    }

    fn push_signed(&mut self, s: &[i8]) {
        let dest = &mut self.data.0[self.len..self.len + s.len()];
        for (i, &b) in s.iter().enumerate() {
            dest[i] = b;
        }
        self.len += s.len();
    }

    /// Get the filled part of the buffer as `&[i8]`.
    fn as_slice(&self) -> &[i8] {
        &self.data.0[..self.len]
    }
}

pub fn get_env(key: &str) -> Option<&'static str> {
    let key_bytes = key.as_bytes();
    assert!(key_bytes.len() < ENV_KEY_SIZE);
    let mut buf = [0i8; ENV_KEY_SIZE];
    // Convert each byte safely
    for (i, &b) in key_bytes.iter().enumerate() {
        buf[i] = b.cast_signed();
    }
    buf[key_bytes.len()] = 0; // null terminator

    let value_ptr = unsafe { libc::getenv(buf.as_ptr()) };
    if value_ptr.is_null() {
        None
    } else {
        let cstr = unsafe { CStr::from_ptr(value_ptr) };
        cstr.to_str().ok()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Instance {
    stream: SocketPath,
    event_socket_path: SocketPath,
}

impl Instance {
    pub fn set(&self, key: &str, value: &str) {
        // Build "keyword {key} {value}" on the stack
        let mut buf = StackBuf::new();
        buf.push(b"keyword ");
        buf.push(key.as_bytes());
        buf.push(b" ");
        buf.push(value.as_bytes());
        self.write_to_socket(buf.as_slice());
    }

    fn write_to_socket(&self, content: &[i8]) {
        let stream = UnixStream::connect(self.stream).unwrap();
        stream.write_all(content).unwrap();
    }

    fn get_hypr_prefix() -> StackBuf {
        let mut buf = StackBuf::new();
        if let Some(xdg) = get_env("XDG_RUNTIME_DIR") {
            buf.push(xdg.as_bytes());
        } else {
            let uid = get_env("UID").expect("Could not find XDG_RUNTIME_DIR or UID");
            buf.push(b"/run/user/");
            buf.push(uid.as_bytes());
        }
        buf.push(b"/hypr");
        buf
    }

    fn get_env_name() -> &'static str {
        get_env("HYPRLAND_INSTANCE_SIGNATURE")
            .expect("Could not get socket path! (Is Hyprland running??)")
    }

    pub fn new() -> Self {
        let mut prefix = Self::get_hypr_prefix();
        prefix.push(b"/");
        prefix.push(Self::get_env_name().as_bytes());
        Self::from_base_socket_path_bytes(prefix.as_slice())
    }

    fn from_base_socket_path_bytes(path: &[i8]) -> Self {
        let mut s1 = StackBuf::new();
        let mut s2 = StackBuf::new();
        s1.push(b"/.socket.sock");
        s2.push(b"/.socket2.sock");

        // Copy the base path into both buffers (they start empty)
        let mut full1 = StackBuf::new();
        let mut full2 = StackBuf::new();
        full1.push(b""); // dummy, but we copy directly
        // Actually easier: construct the full path by pushing the base then the suffix
        let base_slice = path; // already &[i8]
        full1.data.0[..base_slice.len()].copy_from_slice(base_slice);
        full1.len = base_slice.len();
        full1.push_signed(s1.as_slice());

        full2.data.0[..base_slice.len()].copy_from_slice(base_slice);
        full2.len = base_slice.len();
        full2.push_signed(s2.as_slice());

        Self {
            stream: full1.data,
            event_socket_path: full2.data,
        }
    }

    pub fn get_event_stream(&self) -> UnixStream {
        UnixStream::connect(self.event_socket_path).unwrap()
    }
}
