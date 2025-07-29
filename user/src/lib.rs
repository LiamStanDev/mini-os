#![no_std]
#![feature(linkage)]

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    exit(main());
    panic!("unreachable after sys_exit!");
}

#[linkage = "weak"]
#[unsafe(no_mangle)]
fn main() -> i32 {
    panic!("Cannot find main!");
}

use syscall::*;

pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}
pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}
pub fn yield_() -> isize {
    sys_yield()
}
pub fn get_time() -> isize {
    sys_get_time()
}

/// Creates a new process by duplicating current process.
///
/// # Returns
///
/// The child's PID in the parent, and 0 in the child.
pub fn fork() -> isize {
    sys_fork()
}

/// Clear current process address space and load a specified program into it.
///
/// # Arguments
///
/// * `path` - The excutable path.
///
/// Returns
///
/// Returns -1 if error, otherwise no return.
pub fn exec(path: &str) -> isize {
    sys_exec(path)
}

/// Waits for any child process to change state.
///
/// This function blocks the calling process until one of its child processes exits
/// or a signal is received. The exit code of the child process is stored in `exit_code`.
///
/// # Arguments
///
/// * `exit_code` - A mutable reference to an `i32` where the exit code will be stored.
///
/// # Returns
///
/// Returns the PID of the child process that changed state, or -1 on error.
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

/// Waits for a specific child process to change state.
///
/// This function blocks the calling process until the specified child process exits
/// or a signal is received. The exit code of the child process is stored in `exit_code`.
///
/// # Arguments
///
/// * `pid` - The PID of the child process to wait for.
/// * `exit_code` - A mutable reference to an `i32` where the exit code will be stored.
///
/// # Returns
///
/// Returns the PID of the child process that changed state, or -1 on error.
pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            // child not finished yet.
            -2 => {
                yield_();
            }
            // -1 or real pid
            exit_pid => return exit_pid,
        }
    }
}
