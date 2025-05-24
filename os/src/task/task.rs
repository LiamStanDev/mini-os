use super::context::TaskContext;

#[derive(Copy, Clone)]
pub(crate) struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_ctx: TaskContext,
}

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}
