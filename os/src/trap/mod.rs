mod context;

use crate::config::{TRAMPOLINE_ADDR, TRAP_CONTEXT_ADDR};
use crate::syscall::syscall;
use crate::task::{
    current_trap_cx, current_user_token, exit_current_and_run_next, suspend_current_and_run_next,
};
use crate::timer::{self, set_next_trigger};
use core::arch::{asm, global_asm};
use log::info;
use riscv::interrupt::{Exception, Interrupt};
use riscv::register;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Trap},
    stval,
};

global_asm!(include_str!("trap.S"));

/// Initialize the trap handling subsystem.
///
/// This function sets the kernel trap entry point, configuring the hardware to use
/// the appropriate trap vector for handling exceptions, interrupts, and syscalls in S-mode.
pub fn init() {
    set_kernel_trap_entry();
}

/// Set the S-mode trap entry point for the kernel.
///
/// # Note
/// Once the kernel is entered, if another trap occurs in S-mode, the hardware will set some CSR registers
/// and then jump directly to `trap_from_kernel` without saving general-purpose registers.
/// This is because, after separating kernel and user address spaces, the context saving/restoring and trap
/// handling logic for U-mode → S-mode and S-mode → S-mode traps are very different.
/// For simplicity, S-mode → S-mode trap handling is minimized here: we simply panic in `trap_from_kernel`.
fn set_kernel_trap_entry() {
    let mut stvec = register::stvec::read();
    stvec.set_trap_mode(TrapMode::Direct);
    stvec.set_address(trap_from_kernel as usize);
    unsafe {
        register::stvec::write(stvec);
    }
}

/// Set the user trap entry point in stvec to the trampoline address in direct mode.
///
/// This function configures the `stvec` CSR to use direct mode and sets its address to the
/// trampoline code. This ensures that traps from user mode will jump to the trampoline,
/// which is responsible for saving user context and switching to the kernel trap handler.
fn set_user_trap_entry() {
    let mut stvec = register::stvec::read();
    stvec.set_trap_mode(TrapMode::Direct);
    stvec.set_address(TRAMPOLINE_ADDR);
    unsafe {
        register::stvec::write(stvec);
    }
}

/// enable timer interrupt in sie CSR
pub fn enable_timer_interrupt() {
    unsafe {
        riscv::register::sie::set_stimer();
    }

    timer::set_next_trigger();
}

#[unsafe(no_mangle)]
/// handle an interrupt, exception, or system call from user space
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let cx = current_trap_cx();
    let scause = register::scause::read();
    let stval = stval::read();

    let raw_trap: Trap<usize, usize> = scause.cause();
    let standard_trap: Trap<Interrupt, Exception> = raw_trap.try_into().unwrap();
    match standard_trap {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            info!(
                "[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
                stval, cx.sepc
            );
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            info!("[kernel] IllegalInstruction in application, kernel killed it.");
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    trap_return();
}

#[unsafe(no_mangle)]
/// set the new addr of __restore asm function in TRAMPOLINE page,
/// set the reg a0 = trap_cx_ptr, reg a1 = phy addr of usr page table,
/// finally, jump to new addr of __restore asm function
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT_ADDR;
    let user_satp = current_user_token();
    unsafe extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE_ADDR;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",             // jump to new addr of __restore asm function
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,      // a0 = virt addr of Trap Context
            in("a1") user_satp,        // a1 = phy addr of usr page table
            options(noreturn)
        );
    }
}

#[unsafe(no_mangle)]
pub fn trap_from_kernel() -> ! {
    panic!("a trap from kernel!");
}

pub use context::TrapContext;
