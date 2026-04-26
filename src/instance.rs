use crate::{
    SUN_PATH_SIZE,
    buf::Buf,
    unixstream::{SocketPath, UnixStream},
};
use core::ffi::{CStr, c_char};

#[derive(Debug, Clone, Copy)]
pub struct Instance {
    stream: SocketPath,
    event_socket_path: SocketPath,
}

impl Instance {
    pub fn set(&self, key: &str, value: &str) {
        let mut buf = Buf::<SUN_PATH_SIZE>::new();
        buf.push(b"keyword ");
        buf.push(key.as_bytes());
        buf.push(b" ");
        buf.push(value.as_bytes());
        self.write_to_socket(buf.as_signed_slice());
    }

    fn write_to_socket(&self, content: &[i8]) {
        UnixStream::connect(self.stream).unwrap().write_all(content).unwrap();
    }

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

            if let Some(eq_pos) = bytes.iter().position(|&b| b == b'=') {
                let (key_slice, value_slice) = bytes.split_at(eq_pos);
                if key_slice == key_bytes {
                    let value_bytes = &value_slice[1..];
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

    fn get_hypr_prefix(envp: *const *const c_char) -> Buf<SUN_PATH_SIZE> {
        let mut buf = Buf::<SUN_PATH_SIZE>::new();
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
        Self::from_base_socket_path_bytes(prefix.as_signed_slice())
    }

    fn from_base_socket_path_bytes(path: &[i8]) -> Self {
        let mut stream = SocketPath::new();
        let mut event_socket_path = SocketPath::new();
        let len = path.len();
        stream.0[..len].copy_from_slice(path);
        event_socket_path.0[..len].copy_from_slice(path);

        let mut stream_suffix = Buf::<SUN_PATH_SIZE>::new();
        stream_suffix.push(b"/.socket.sock");
        stream.0[len..len + stream_suffix.len].copy_from_slice(stream_suffix.as_signed_slice());

        let mut event_suffix = Buf::<SUN_PATH_SIZE>::new();
        event_suffix.push(b"/.socket2.sock");
        event_socket_path.0[len..len + event_suffix.len].copy_from_slice(event_suffix.as_signed_slice());

        Self {
            stream,
            event_socket_path,
        }
    }

    pub fn get_event_stream(&self) -> UnixStream {
        UnixStream::connect(self.event_socket_path).unwrap()
    }
}
