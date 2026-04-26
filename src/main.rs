#![allow(internal_features)]
#![feature(lang_items, core_intrinsics)]
#![no_std]
#![no_main]
mod event;
mod instance;
mod unixstream;

extern crate libc;

use crate::{event::ActiveWindowChangedEventListener, instance::Instance};
use core::{
    cell::Cell,
    ffi::{CStr, c_char, c_int, c_void},
    fmt::Write,
    panic::PanicInfo,
};

static mut ARGS_BUF: Buf<512> = Buf::new();
const EVENT_BUFFER_SIZE: usize = 256; // arbitrary
const ENV_KEY_SIZE: usize = 64; // $HYPRLAND_INSTANCE_SIGNATURE is 63 bytes + null terminator
const SUN_PATH_SIZE: usize = 108; // size of sockaddr_un.sun_path

#[link(name = "c")]
unsafe extern "C" {}

#[link(name = "gcc_s")]
unsafe extern "C" {}

unsafe extern "C" {
    fn write(fd: i32, buf: *const c_void, count: usize) -> isize;
}

fn __write(fd: i32, msg: &str) {
    let n = unsafe { write(fd, msg.as_ptr().cast::<c_void>(), msg.len()) };

    if n < 0 {
        unsafe { libc::exit(7) }; // We don't want to have to return from every function
    }
}

fn print(msg: &str) {
    __write(1, msg);
}

struct Buf<const N: usize> {
    data: [u8; N],
    len: usize,
}

impl<const N: usize> Buf<N> {
    const fn new() -> Self {
        Self {
            data: [0; N],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        // Safety: we only ever write valid UTF-8 via write_str.
        unsafe { core::str::from_utf8_unchecked(&self.data[..self.len]) }
    }

    const fn clear(&mut self) {
        self.len = 0;
    }
}

impl<const N: usize> Write for Buf<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let b = s.as_bytes();
        if self.len + b.len() > N {
            return Err(core::fmt::Error);
        }
        self.data[self.len..self.len + b.len()].copy_from_slice(b);
        self.len += b.len();
        Ok(())
    }
}

// kitty, SUPER, q, exec, uwsm app -- kitty
#[unsafe(no_mangle)]
unsafe extern "C" fn main(argc: usize, argv: *const *const c_char) -> c_int {
    // Build the joined argv string into the static buffer.
    let rest: &'static str = unsafe {
        let buf = &mut *core::ptr::addr_of_mut!(ARGS_BUF);
        for i in 1..argc {
            let arg_ptr = *argv.add(i);
            if !arg_ptr.is_null() {
                let cstr = CStr::from_ptr(arg_ptr);
                if let Ok(s) = cstr.to_str() {
                    if buf.len > 0 {
                        buf.write_str(" ").ok();
                    }
                    buf.write_str(s).ok();
                }
            }
        }
        buf.as_str()
    };

    let (class, rest): (&'static str, &'static str) = rest.split_once(", ").unwrap();
    let (modifiers, rest): (&'static str, &'static str) = rest.split_once(", ").unwrap();
    let (key, action): (&'static str, &'static str) = rest.split_once(", ").unwrap();

    let is_bind_set = Cell::new(false);
    let instance = Instance::new();

    ActiveWindowChangedEventListener(move |wevent| {
        let should_bind_be_set = wevent.class != class;
        if should_bind_be_set == is_bind_set.get() {
            return;
        }

        let mut buf = Buf::<128>::new();
        if should_bind_be_set {
            is_bind_set.set(true);
            write!(buf, "{modifiers},{key},{action}").ok();
            instance.set("bind", buf.as_str());
            buf.clear();
            writeln!(buf, "keyword bind {modifiers}, {key}, {action}").ok();
        } else {
            is_bind_set.set(false);
            write!(buf, "{modifiers},{key}").ok();
            instance.set("unbind", buf.as_str());
            buf.clear();
            writeln!(buf, "keyword unbind {modifiers}, {key}").ok();
        }
        print(buf.as_str());
    })
    .start(&instance);

    0
}

#[lang = "eh_personality"]
const fn rust_eh_personality() {}
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    let mut buf = Buf::<256>::new();
    writeln!(buf, "{info}").ok();
    print(buf.as_str());
    core::intrinsics::abort()
}
