use core::arch::global_asm;

use self::context::TrapContext;
use crate::syscall::syscall;
use crate::task::suspend_current_and_run_next;
use crate::timer::{self, set_next_trigger};
use log::trace;
use riscv::interrupt::{Exception, Interrupt, Trap};
use riscv::register;
use riscv::register::stvec::TrapMode;

pub(crate) mod context;

global_asm!(include_str!("trap.S"));

// set stvec register to trap entry point
pub(crate) fn init() {
    unsafe extern "C" {
        safe fn __alltraps();
    }
    let mut stvec = register::stvec::read();
    stvec.set_address(__alltraps as usize);
    stvec.set_trap_mode(TrapMode::Direct);
    unsafe {
        register::stvec::write(stvec);
    }
}

pub(crate) fn enable_timer_interrupt() {
    unsafe {
        // enable supervisor timer interrupt
        riscv::register::sie::set_stimer();
    }
    timer::set_next_trigger();
}

#[unsafe(no_mangle)]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
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
            cx.sepc += 4; // ecall has 4 bytes. sepc is the sret's return address 
            // x10: a0, x17: a7, x10: a0, x11: a1
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize
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
    cx
}
