use crate::{
    unixstream::{SocketPath, UnixStream},
};
use core::ffi::{CStr, c_char};

pub struct StackBuf {
    pub data: SocketPath,
    pub len: usize,
}

impl StackBuf {
    pub const fn new() -> Self {
        Self {
            data: SocketPath::new(),
            len: 0,
        }
    }

    /// Safely push a slice of `u8` bytes (e.g. from `&str`) onto the buffer.
    pub fn push(&mut self, s: &[u8]) {
        let dest = &mut self.data.0[self.len..self.len + s.len()];
        for (i, &b) in s.iter().enumerate() {
            dest[i] = b.cast_signed();
        }
        self.len += s.len();
    }

    pub fn push_signed(&mut self, s: &[i8]) {
        let dest = &mut self.data.0[self.len..self.len + s.len()];
        for (i, &b) in s.iter().enumerate() {
            dest[i] = b;
        }
        self.len += s.len();
    }

    /// Get the filled part of the buffer as `&[i8]`.
    pub fn as_slice(&self) -> &[i8] {
        &self.data.0[..self.len]
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

    /// Retrieves an environment variable's value from the given `envp` array.
    ///
    /// # Safety
    /// `envp` must be a valid pointer to a null‑terminated array of null‑terminated
    /// C strings. Each string must be in the form `"KEY=value"`.
    ///
    /// # Panics
    /// - If `key` is not found in the environment.
    /// - If any environment value contains invalid UTF‑8 (use `from_utf8_unchecked`
    ///   to avoid this check if you know the values are UTF‑8).
    #[must_use]
    unsafe fn get_env(envp: *const *const c_char, key: &'static str) -> Option<&'static str> {
        let key_bytes = key.as_bytes();
        let mut i = 0;

        loop {
            let entry_ptr = unsafe { *envp.add(i) };
            if entry_ptr.is_null() {
                break;
            }

            let cstr = unsafe { CStr::from_ptr(entry_ptr) };
            let bytes = cstr.to_bytes();

            // Find the first '=' separator
            if let Some(eq_pos) = bytes.iter().position(|&b| b == b'=') {
                let (key_slice, value_slice) = bytes.split_at(eq_pos);
                // key_slice does not include '=', value_slice starts after '='
                if key_slice == key_bytes {
                    // The value part starts after the '='
                    let value_bytes = &value_slice[1..];
                    // Convert to &str (panics if not valid UTF‑8)
                    return Some(
                        str::from_utf8(value_bytes)
                            .expect("environment variable value is not valid UTF-8"),
                    );
                }
            }
            i += 1;
        }

        None
    }

    fn get_env_name(envp: *const *const c_char) -> &'static str {
        unsafe { Self::get_env(envp, "HYPRLAND_INSTANCE_SIGNATURE") }
            .expect("Could not get socket path! (Is Hyprland running??)")
    }

    fn get_hypr_prefix(envp: *const *const c_char) -> StackBuf {
        let mut buf = StackBuf::new();
        buf.push(unsafe {
            Self::get_env(envp, "XDG_RUNTIME_DIR")
                .expect("Could not find $XDG_RUNTIME_DIR")
                .as_bytes()
        });
        buf.push(b"/hypr");
        buf
    }

    pub fn new(envp: *const *const c_char) -> Self {
        let mut prefix = Self::get_hypr_prefix(envp);
        prefix.push(b"/");
        prefix.push(Self::get_env_name(envp).as_bytes());
        Self::from_base_socket_path_bytes(prefix.as_slice())
    }

    pub fn from_base_socket_path_bytes(path: &[i8]) -> Self {
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
