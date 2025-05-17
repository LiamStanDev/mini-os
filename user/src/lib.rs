#![no_std]
#![feature(linkage)]

mod lang_items;
mod syscall;
#[macro_use]
pub mod console;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    clear_bss();
    exit(main());
    panic!("unreachable after sys_exist!");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
fn main() -> i32 {
    panic!("Cannot find main");
}

fn clear_bss() {
    unsafe extern "C" {
        safe fn start_bss();
        safe fn end_bss();
    }

    (start_bss as usize..end_bss as usize).for_each(|addr| unsafe {
        (addr as *mut u8).write_volatile(0);
    });
}

use syscall::*;

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}
