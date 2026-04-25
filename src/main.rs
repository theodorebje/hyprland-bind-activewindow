#![allow(internal_features)]
#![feature(lang_items, core_intrinsics)]
#![no_std]
#![no_main]
mod bufreader;
mod event;
mod instance;
mod unixstream;

extern crate alloc;
extern crate libc;

use crate::{event::ActiveWindowChangedEventListener, instance::Instance};
use alloc::{boxed::Box, format, string::ToString, vec::Vec};
use core::{
    cell::Cell,
    ffi::{CStr, c_char, c_int, c_void},
    panic::PanicInfo,
};

#[link(name = "c")]
unsafe extern "C" {}

#[link(name = "gcc_s")]
unsafe extern "C" {}

// Choose a heap size (4 KiB in this example).
// 8 bytes overhead, must be ≥8 and divisible by 4.
#[global_allocator]
static ALLOCATOR: emballoc::Allocator<4096> = emballoc::Allocator::new();

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

// kitty, SUPER, q, exec, uwsm app -- kitty
#[unsafe(no_mangle)]
unsafe extern "C" fn main(argc: usize, argv: *const *const c_char) -> c_int {
    // Convert raw arguments into Vec<String>.
    let args = unsafe {
        let mut vec = Vec::new();
        for i in 1..argc {
            // skip argv[0] (program name)
            let arg_ptr = *argv.add(i);
            if !arg_ptr.is_null() {
                let cstr = CStr::from_ptr(arg_ptr);
                if let Ok(s) = cstr.to_str() {
                    vec.push(s.to_string());
                }
            }
        }
        vec
    };

    let rest: &'static str = Box::leak(args.join(" ").into_boxed_str());
    let (class, rest): (&'static str, &'static str) = rest.split_once(", ").unwrap();
    let (modifiers, rest): (&'static str, &'static str) = rest.split_once(", ").unwrap();
    let (key, action): (&'static str, &'static str) = rest.split_once(", ").unwrap();

    let is_bind_set = Cell::new(false);

    let instance = Instance::new();
    let i2 = instance.clone();

    ActiveWindowChangedEventListener(move |wevent| {
        let should_bind_be_set = wevent.class != class;
        if should_bind_be_set == is_bind_set.get() {
            return;
        }
        if should_bind_be_set {
            is_bind_set.set(true);
            i2.set("bind", &format!("{modifiers},{key},{action}"));
            print(&format!("keyword bind {modifiers}, {key}, {action}\n"));
        } else {
            is_bind_set.set(false);
            i2.set("unbind", &format!("{modifiers},{key}"));
            print(&format!("keyword unbind {modifiers}, {key}\n"));
        }
    })
    .start(&instance);
    0
}

#[lang = "eh_personality"]
const fn rust_eh_personality() {}
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    print(&format!("{info}\n"));

    core::intrinsics::abort()
}
