#![allow(internal_features)]
#![feature(core_intrinsics, c_size_t)]
#![no_std]
#![no_main]
mod buf;
mod bufstream;
mod event;
mod instance;
mod libasm;
mod unixstream;

use crate::{buf::Buf, event::ActiveWindowChangedEventListener, instance::Instance, libasm::exit};
use core::{
    cell::Cell,
    ffi::{CStr, c_char, c_int},
    fmt::Write,
    panic::PanicInfo,
};
use crate::libasm::print;

const DEBUG_OUT_SIZE: usize = 256;
const ARGS_BUF_SIZE: usize = 256;
const EVENT_BUFFER_SIZE: usize = 256;
const SUN_PATH_SIZE: usize = 108;
static mut ARGS_BUF: Buf<ARGS_BUF_SIZE> = Buf::new();

fn main(
    argc: usize,
    argv: *const *const c_char,
    envp: *const *const c_char,
) -> c_int {
    let rest: &'static str = unsafe {
        let buf = &mut *core::ptr::addr_of_mut!(ARGS_BUF);
        buf.clear();
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
    let instance = Instance::new(envp);

    ActiveWindowChangedEventListener(move |wevent| {
        let should_bind_be_set = wevent.class != class;
        if should_bind_be_set == is_bind_set.get() {
            return;
        }

        let mut buf = Buf::<DEBUG_OUT_SIZE>::new();
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

    exit(0)
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        "mov rax, rsp",
        "mov rdi, [rax]",
        "lea rsi, [rax + 8]",
        "lea rdx, [rsi + rdi * 8 + 8]",
        "sub rsp, 8",
        "jmp {runtime_entry}",
        runtime_entry = sym main,
    );
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    let mut buf = Buf::<256>::new();
    writeln!(buf, "{info}").ok();
    print(buf.as_str());
    core::intrinsics::abort()
}
