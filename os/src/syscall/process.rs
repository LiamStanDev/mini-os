use log::*;

use crate::task::*;

pub(crate) fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);

    exit_current_and_run_next();
    panic!("unreachable in sys_exit");
}

pub(crate) fn sys_yield() -> isize {
    trace!("[kernel] Application yield");
    suspend_current_and_run_next();
    0
}
