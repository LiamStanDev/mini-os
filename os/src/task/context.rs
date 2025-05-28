use crate::trap::trap_return;

#[repr(C)]
#[derive(Copy, Clone)]
/// The task context structure used for task switching.
///
/// `TaskContext` saves the minimal set of registers required to resume a task after a context switch.
/// This typically includes the return address (`ra`), stack pointer (`sp`), and callee-saved registers (`s`).
pub struct TaskContext {
    /// Return address register (ra, x1).
    pub ra: usize,
    /// Stack pointer register (sp, x2).
    pub sp: usize,
    /// Callee-saved registers (s0-s11, x8-x9, x18-x27).
    pub s: [usize; 12],
}

impl TaskContext {
    pub fn empty() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }

    /// Create a new `TaskContext` for returning from a trap handler.
    ///
    /// This function initializes a `TaskContext` such that, when scheduled,
    /// it will jump to the `trap_return` routine with the given kernel stack pointer.
    /// All callee-saved registers are zero-initialized.
    ///
    /// # Arguments
    /// * `kstack_ptr` - The kernel stack pointer to use when returning from the trap.
    ///
    /// # Returns
    /// A `TaskContext` set up to return from a trap handler.
    pub fn goto_trap_return(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}
