use crate::unixstream::UnixStream;
use alloc::{
    ffi::CString,
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::{ffi::CStr, fmt::Write};

pub fn get_env(key: &str) -> Option<String> {
    // Convert the key to a null‑terminated C string
    let key_cstr = CString::new(key).ok()?;
    unsafe {
        let value_ptr = libc::getenv(key_cstr.as_ptr());
        if value_ptr.is_null() {
            None
        } else {
            // The returned pointer points to static data; copy it into a String
            let cstr = CStr::from_ptr(value_ptr);
            Some(cstr.to_str().ok()?.to_string())
        }
    }
}

/// This is the sync version of the Hyprland Instance.
/// It holds the event streams connected to the sockets of one running Hyprland instance.
#[derive(Debug, Clone)]
pub struct Instance {
    /// .socket.sock
    stream: String,
    /// .socket2.sock
    event_socket_path: String,
}

impl Instance {
    /// This function sets a keyword's value
    pub fn set(&self, key: &str, value: &str) {
        self.write_to_socket(&format!("keyword {key} {value}"));
    }

    fn write_to_socket(&self, content: &str) {
        let stream = UnixStream::connect(&self.stream).unwrap();
        stream.write_all(content.as_bytes()).unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();
    }

    fn get_hypr_path() -> String {
        let mut buf = get_env("XDG_RUNTIME_DIR").map_or_else(
            || {
                let uid = get_env("UID").expect("Could not find XDG_RUNTIME_DIR or UID");
                format!("/run/user/{uid}")
            },
            String::from,
        );
        buf.push_str("/hypr");
        buf
    }

    fn get_env_name() -> String {
        get_env("HYPRLAND_INSTANCE_SIGNATURE")
            .expect("Could not get socket path! (Is Hyprland running??)")
    }

    pub fn new() -> Self {
        let mut path = Self::get_hypr_path();
        let name = Self::get_env_name();
        let _ = write!(path, "/{name}");
        Self::from_base_socket_path(&path)
    }

    /// Uses the path to determine the sockets to use
    ///
    /// Example path: `/run/user/1000/hypr/9958d297641b5c84dcff93f9039d80a5ad37ab00_1752788564_21468021`
    pub fn from_base_socket_path(path: &str) -> Self {
        Self {
            stream: format!("{path}/.socket.sock"),
            event_socket_path: format!("{path}/.socket2.sock"),
        }
    }

    pub fn get_event_stream(&self) -> UnixStream {
        UnixStream::connect(&self.event_socket_path).unwrap()
    }
}
