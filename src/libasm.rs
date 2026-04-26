use crate::{
    libasm::syscall::{sockaddr, sockaddr_un, socklen_t},
    unixstream::SocketPath,
};
use core::ffi::{c_int, c_size_t, c_ushort, c_void};

pub const EINVAL: c_int = -22;
pub const EPIPE: c_int = -32;
const STDOUT_FILENO: c_int = 1;
const AUTOMATIC_PROTOCOL: c_int = 0;
const SOCK_STREAM: c_int = 1;
const AF_UNIX: c_ushort = 1;
const AF_UNIX_INT: c_int = AF_UNIX as c_int;

mod syscall {
    use crate::{SUN_PATH_SIZE, libasm::AF_UNIX, unixstream::SocketPath};
    use core::{
        arch::asm,
        ffi::{c_char, c_int, c_size_t, c_ssize_t, c_uint, c_ushort, c_void},
    };

    const SYS_SOCKET: c_int = 41;
    const SYS_CONNECT: c_int = 42;
    const SYS_CLOSE: c_int = 3;
    const SYS_READ: c_int = 0;
    const SYS_WRITE: c_int = 1;
    const SYS_EXIT: c_int = 60;

    #[allow(non_camel_case_types)]
    pub type socklen_t = c_uint;

    #[allow(non_camel_case_types)]
    #[repr(C)]
    pub struct sockaddr {
        pub sa_family: c_ushort,
        pub sa_data: [c_char; 14],
    }

    #[allow(non_camel_case_types)]
    #[repr(C)]
    pub struct sockaddr_un {
        pub sun_family: c_ushort,
        pub sun_path: SocketPath,
    }

    fn socket_path_len(path: SocketPath) -> usize {
        path.0.iter().position(|&c| c == 0).unwrap_or(SUN_PATH_SIZE)
    }

    impl sockaddr_un {
        pub fn new(path: SocketPath) -> Option<(Self, c_size_t)> {
            // Find the length of the null‑terminated string inside sun_path
            // SAFETY: The user guarantees that `path.0` contains a valid null‑terminated C string
            //         of length at most 107 (leaving room for the null terminator).
            let path_len = socket_path_len(path);
            // Ensure the path is not longer than the buffer (should never happen with correct input)
            if path_len >= path.0.len() {
                return None;
            }

            Some((
                Self {
                    sun_family: AF_UNIX,
                    sun_path: path,
                },
                path_len,
            ))
        }
    }

    pub unsafe fn socket(domain: c_int, ty: c_int, protocol: c_int) -> c_int {
        let ret: c_int;
        unsafe {
            asm!(
                "syscall",
                in("rax") SYS_SOCKET,
                in("rdi") domain,
                in("rsi") ty,
                in("rdx") protocol,
                lateout("rcx") _,   // clobbered by syscall
                lateout("r11") _,   // clobbered by syscall
                lateout("rax") ret,
                options(nostack, preserves_flags)
            );
        };
        ret
    }

    pub unsafe fn connect(socket: c_int, address: *const sockaddr, len: socklen_t) -> c_int {
        let ret: c_int;
        unsafe {
            asm!(
                "syscall",
                in("rax") SYS_CONNECT,
                in("rdi") socket,
                in("rsi") address,
                in("rdx") len,
                lateout("rcx") _,
                lateout("r11") _,
                lateout("rax") ret,
                options(nostack, preserves_flags)
            );
        };
        ret
    }

    pub unsafe fn read(fd: c_int, buf: *mut c_void, count: c_size_t) -> c_ssize_t {
        let ret: c_ssize_t;
        unsafe {
            asm!(
                "syscall",
                in("rax") SYS_READ,
                in("rdi") fd,
                in("rsi") buf,
                in("rdx") count,
                lateout("rcx") _,
                lateout("r11") _,
                lateout("rax") ret,
                options(nostack, preserves_flags)
            );
        };
        ret
    }

    pub unsafe fn write(fd: c_int, buf: *const c_void, count: c_size_t) -> c_ssize_t {
        let ret: c_ssize_t;
        unsafe {
            asm!(
                "syscall",
                in("rax") SYS_WRITE,
                in("rdi") fd,
                in("rsi") buf,
                in("rdx") count,
                lateout("rcx") _,
                lateout("r11") _,
                lateout("rax") ret,
                options(nostack, preserves_flags)
            );
        };
        ret
    }

    pub unsafe fn close(fd: c_int) -> c_int {
        let ret: c_int;
        unsafe {
            asm!(
                "syscall",
                in("rax") SYS_CLOSE,
                in("rdi") fd,
                lateout("rcx") _,
                lateout("r11") _,
                lateout("rax") ret,
                options(nostack, preserves_flags)
            );
        };
        ret
    }

    pub unsafe fn exit(status: c_int) -> ! {
        unsafe {
            asm!(
                "syscall",
                in("rax") SYS_EXIT,
                in("rdi") status,
                options(noreturn, nostack)
            )
        };
    }
}

pub fn exit(status: c_int) -> ! {
    unsafe { syscall::exit(status) }
}

pub fn create_unix_socket() -> Result<c_int, c_int> {
    let ret = unsafe { syscall::socket(AF_UNIX_INT, SOCK_STREAM, AUTOMATIC_PROTOCOL) };

    if ret < 0 {
        Err(ret) // syscall returns errno
    } else {
        Ok(ret)
    }
}

pub fn close(fd: c_int) -> Result<c_int, c_int> {
    let ret = unsafe { syscall::close(fd) };

    if ret < 0 { Err(ret) } else { Ok(ret) }
}

fn write(fd: c_int, buf: *const c_void, count: c_size_t) -> Result<c_size_t, c_int> {
    let ret = unsafe { syscall::write(fd, buf, count) };

    if ret < 0 {
        Err(c_int::try_from(ret).unwrap())
    } else {
        Ok(ret.cast_unsigned())
    }
}

pub fn write_str(fd: c_int, msg: &str) -> Result<c_size_t, c_int> {
    write(fd, msg.as_ptr().cast::<c_void>(), msg.len())
}

pub fn write_signed(fd: c_int, buf: &[i8]) -> Result<c_size_t, c_int> {
    write(fd, buf.as_ptr().cast::<c_void>(), buf.len())
}

pub fn print(msg: &str) {
    write_str(STDOUT_FILENO, msg).expect("todo");
}

pub fn read(fd: c_int, buf: &mut [u8]) -> Result<c_size_t, c_int> {
    let ret = unsafe { syscall::read(fd, buf.as_mut_ptr().cast::<c_void>(), buf.len()) };
    if ret < 0 {
        Err(c_int::try_from(ret).unwrap())
    } else {
        Ok(ret.cast_unsigned())
    }
}

fn connect(fd: c_int, addr: &sockaddr_un, path_len: c_size_t) -> Result<c_int, c_int> {
    let addr_ptr = core::ptr::from_ref::<sockaddr_un>(addr).cast::<sockaddr>();

    let addr_len =
        socklen_t::try_from(core::mem::offset_of!(sockaddr_un, sun_path) + path_len + 1).unwrap();

    let ret = unsafe { syscall::connect(fd, addr_ptr, addr_len) };

    if ret < 0 { Err(ret) } else { Ok(ret) }
}

pub fn connect_to_socket(path: SocketPath) -> Result<c_int, c_int> {
    let fd = create_unix_socket()?;
    let Some((addr, path_len)) = sockaddr_un::new(path) else {
        let _ = close(fd);
        return Err(EINVAL);
    };

    match connect(fd, &addr, path_len) {
        Ok(_) => Ok(fd),
        Err(err) => {
            let _ = close(fd);
            Err(err)
        }
    }
}
