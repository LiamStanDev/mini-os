core::arch::global_asm!(include_str!("switch.S"));
use super::TaskContext;

unsafe extern "C" {
    /// Externally defined low-level context switch routine for tasks.
    ///
    /// The `__switch` function saves the current task's context (registers) into `current_task_ctx_ptr`
    /// and restores the next task's context from `next_task_ctx_ptr`. This function is implemented in
    /// assembly and is essential for preemptive multitasking.
    ///
    /// # Safety
    /// This function is unsafe because it performs raw pointer dereferencing and low-level context switching.
    pub unsafe fn __switch(
        current_task_cx_ptr: *mut TaskContext,
        next_task_cx_ptr: *const TaskContext,
    );
}
