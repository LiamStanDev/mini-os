use crate::loader;
use crate::sbi::shutdown;
use crate::sync::UPSafeCell;
use crate::task::context::TaskContext;
use crate::trap::context::TrapContext;
use alloc::vec::Vec;
use lazy_static::*;
use log::trace;

use self::switch::__switch;
use self::task::{TaskControlBlock, TaskStatus};

mod context;
mod switch;
mod task;

pub struct TaskManager {
    num_app: usize,                      // unchange
    inner: UPSafeCell<TaskManagerInner>, // change when running
}

impl TaskManager {
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        trace!("task {current} suspended");
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        trace!("task {current} exited");
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    // return next app_id
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|app_id| app_id % self.num_app)
            .find(|app_id| inner.tasks[*app_id].task_status == TaskStatus::Ready)
    }

    fn run_next_task(&self) {
        if let Some(next_app_id) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            trace!("task {current} start");
            inner.tasks[next_app_id].task_status = TaskStatus::Running;
            inner.current_task = next_app_id;
            let current_task_ctx_ptr = &mut inner.tasks[current].task_ctx as *mut TaskContext;
            let next_task_ctx_ptr = &inner.tasks[next_app_id].task_ctx as *const TaskContext;
            drop(inner); // switch will modify inner

            // switch
            unsafe {
                __switch(current_task_ctx_ptr, next_task_ctx_ptr);
            }
        } else {
            trace!("All applications completed!");
            shutdown(false);
        }
    }

    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let first_task_ctx_ptr = &task0.task_ctx as *const TaskContext;
        drop(inner);

        let mut dummy = TaskContext::empty();

        unsafe {
            __switch(&mut dummy as *mut TaskContext, first_task_ctx_ptr);
        }

        panic!("unreachable in run_first_task!");
    }

    fn get_current_satp(&self) -> usize {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].satp()
    }

    fn get_current_trap_ctx_mut(&self) -> &'static mut TrapContext {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].get_trap_ctx_mut()
    }
}

struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    current_task: usize,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        trace!("init TASK_MANAGER");
        let num_app = loader::get_num_app();
        trace!("num_app = {}", num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for app_id in 0..num_app {
            tasks.push(TaskControlBlock::new(app_id, loader::get_app_data(app_id)));
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

pub fn suspend_current_and_run_next() {
    TASK_MANAGER.mark_current_suspended();
    TASK_MANAGER.run_next_task();
}

pub fn exit_current_and_run_next() {
    TASK_MANAGER.mark_current_exited();
    TASK_MANAGER.run_next_task();
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

pub fn current_satp() -> usize {
    TASK_MANAGER.get_current_satp()
}

pub fn current_trap_ctx_mut() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_ctx_mut()
}
