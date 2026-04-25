use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
};

/// This is the sync version of the Hyprland Instance.
/// It holds the event streams connected to the sockets of one running Hyprland instance.
#[derive(Debug, Clone)]
pub struct Instance {
    /// .socket.sock
    stream: Box<Path>,
    /// .socket2.sock
    event_socket_path: Box<Path>,
}

impl Instance {
    /// This function sets a keyword's value
    pub fn set(&self, key: &str, value: String) {
        self.write_to_socket(format!("keyword {key} {value}"))
    }

    fn write_to_socket(&self, content: String) {
        let mut stream = UnixStream::connect(&self.stream).unwrap();
        stream.write_all(content.as_bytes()).unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();
    }

    fn get_hypr_path() -> PathBuf {
        let mut buf = if let Some(runtime_path) = std::env::var_os("XDG_RUNTIME_DIR") {
            PathBuf::from(runtime_path)
        } else if let Ok(uid) = std::env::var("UID") {
            PathBuf::from("/run/user/".to_owned() + &uid)
        } else {
            panic!("Could not find XDG_RUNTIME_DIR or UID");
        };
        buf.push("hypr");
        buf
    }

    fn get_env_name() -> String {
        match std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
            Ok(var) => var,
            Err(std::env::VarError::NotPresent) => {
                panic!("Could not get socket path! (Is Hyprland running??)")
            }
            Err(std::env::VarError::NotUnicode(_)) => {
                panic!("Corrupted Hyprland socket variable: Invalid unicode!")
            }
        }
    }

    pub fn new() -> Instance {
        let mut path = Self::get_hypr_path();
        let name = Self::get_env_name();
        path.push(&name);
        Instance::from_base_socket_path(path)
    }

    /// Uses the path to determine the sockets to use
    ///
    /// Example path: `/run/user/1000/hypr/9958d297641b5c84dcff93f9039d80a5ad37ab00_1752788564_21468021`
    pub fn from_base_socket_path(path: PathBuf) -> Self {
        if !path.exists() {
            panic!("Hyprland instance path does not exist: {}", path.display());
        }
        Self {
            stream: path.join(".socket.sock").into_boxed_path(),
            event_socket_path: path.join(".socket2.sock").into_boxed_path(),
        }
    }

    pub fn get_event_stream(&self) -> UnixStream {
        UnixStream::connect(&self.event_socket_path).unwrap()
    }
}
