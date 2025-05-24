use core::arch::global_asm;

use super::context::TaskContext;

global_asm!(include_str!("switch.S"));

unsafe extern "C" {
    // NOTE: why current use mut, but next not?
    // switch function will save(write) registers into current_task_ctx which is in kernel stack, then
    // restore(read) registers from next_task_ctx_ptr.
    pub fn __switch(current_task_ctx_ptr: *mut TaskContext, next_task_ctx_ptr: *const TaskContext);
}
