use crate::trap::context::TrapContext;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 12], // callee save register
}

impl TaskContext {
    // initialize task context
    pub(crate) fn default() -> Self {
        TaskContext {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }

    pub(crate) fn init(kstack_ptr: &'static TrapContext) -> Self {
        unsafe extern "C" {
            fn __restore();
        }

        Self {
            ra: __restore as usize,
            sp: kstack_ptr as *const _ as usize,
            s: [0; 12],
        }
    }
}
