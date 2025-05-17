use log::*;

use crate::batch::run_next_app;

pub(crate) fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    run_next_app()
}
