#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![allow(unused)]

extern crate alloc;

#[macro_use]
extern crate bitflags;

use log::*;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod config;
mod lang_items;
mod loader;
mod logging;
mod mm;
mod sbi;
mod sync;
pub mod syscall;
pub mod task;
mod timer;
pub mod trap;

core::arch::global_asm!(include_str!("entry.asm"));
core::arch::global_asm!(include_str!("link_app.S"));

unsafe extern "C" {
    pub(crate) safe fn stext();
    pub(crate) safe fn etext();
    pub(crate) safe fn srodata();
    pub(crate) safe fn erodata();
    pub(crate) safe fn sdata();
    pub(crate) safe fn edata();
    pub(crate) safe fn sbss_with_stack();
    pub(crate) safe fn sbss();
    pub(crate) safe fn ebss();
    pub(crate) safe fn ekernel();
    pub(crate) safe fn strampoline();
}

/// clear BSS segment
fn clear_bss() {
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

/// the rust entry-point of os
#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    logging::init();
    info!("[kernel] Hello, world!");
    mm::init();
    info!("[kernel] back to world!");
    trap::init();
    trap::enable_timer_interrupt();
    task::run_first_task();
    panic!("Unreachable in rust_main!");
}
