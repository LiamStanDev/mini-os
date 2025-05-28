use crate::config::{TRAMPOLINE_ADDR, TRAP_CONTEXT_ADDR};
use crate::syscall::syscall;
use crate::task::{current_satp, current_trap_ctx_mut, suspend_current_and_run_next};
use crate::timer::{self, set_next_trigger};
use core::arch::{asm, global_asm};
use log::trace;
use riscv::interrupt::{Exception, Interrupt, Trap};
use riscv::register;
use riscv::register::stvec::TrapMode;

pub mod context;

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
    stvec.set_address(TRAMPOLINE_ADDR as usize);
    unsafe {
        register::stvec::write(stvec);
    }
}

pub fn enable_timer_interrupt() {
    unsafe {
        // enable supervisor timer interrupt
        riscv::register::sie::set_stimer();
    }
    timer::set_next_trigger();
}

#[unsafe(no_mangle)]
pub fn trap_from_kernel() -> ! {
    panic!("a trap from kernel!");
}

#[unsafe(no_mangle)]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let ctx = current_trap_ctx_mut();
    // see: https://docs.rs/riscv/latest/riscv/interrupt/enum.Trap.html#example
    let scause = register::scause::read();
    let raw_trap: Trap<usize, usize> = scause.cause();
    let standard_trap: Trap<Interrupt, Exception> = raw_trap.try_into().unwrap();
    match standard_trap {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            trace!("[kernel] SupervisorTimer interrupt");
            set_next_trigger();
            suspend_current_and_run_next();
        }
        Trap::Exception(Exception::UserEnvCall) => {
            ctx.sepc += 4; // ecall has 4 bytes. sepc is the sret's return address 
            // x10: a0, x17: a7, x10: a0, x11: a1
            ctx.x[10] = syscall(ctx.x[17], [ctx.x[10], ctx.x[11], ctx.x[12]]) as usize
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::LoadFault) => {
            trace!("[kernel] PageFault in application, kernel killed it.");
            panic!("[kernel] Cannot continue!");
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            trace!("[kernel] IllegalInstruction in application, kernel killed it.");
            panic!("[kernel] Cannot continue!");
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                standard_trap,
                register::stval::read() // other trap info
            )
        }
    }
    trap_return();
}

/// Return from a trap handler to user mode by restoring user context and switching address space.
///
/// This function sets up the trap entry for user mode, prepares the trap context pointer and user SATP,
/// and jumps to the `__restore` trampoline routine. The trampoline restores all user registers and
/// returns to user mode using `sret`. This function does not return; it will panic if reached.
///
/// # Panics
/// This function will panic if control returns after the trampoline, which should be unreachable.
#[unsafe(no_mangle)]
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_ctx_ptr = TRAP_CONTEXT_ADDR;
    let user_satp = current_satp();
    unsafe extern "C" {
        fn __alltraps();
        fn __restore();
    }

    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE_ADDR;
    unsafe {
        asm!(
            "fence.i", // prevent wrong i-cache
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_ctx_ptr,
            in("a1") user_satp
        );
    }

    panic!("Unreachable in back_to_user!");
}
