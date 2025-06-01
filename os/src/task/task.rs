use super::TaskContext;
use crate::config::{TRAP_CONTEXT_ADDR, kernel_stack_pos};
use crate::mm::{KERNEL_SPACE, MapPermission, MemorySet, PhysPageNum, VirtAddr};
use crate::trap::{TrapContext, trap_handler};

/// The TaskControlBlock holds all information needed to manage and schedule a task.
///
/// Fields:
/// - `task_status`: The current status of the task (e.g., Ready, Running, Exited).
/// - `task_ctx`: The saved CPU context for context switching.
/// - `memory_set`: The address space and memory mappings for the task.
/// - `trap_ctx_ppn`: The physical page number of the trap context for this task.
/// - `base_size`: The size of the application from address 0x0 to the top of the user stack.
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub memory_set: MemorySet,
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
}

impl TaskControlBlock {
    /// Create a new `TaskControlBlock` from an ELF binary and application ID.
    ///
    /// This function sets up the address space, kernel/user stacks, and trap context
    /// for a new user application. It loads the ELF, allocates the kernel stack,
    /// initializes the trap context, and prepares the task for scheduling.
    ///
    /// # Arguments
    /// * `elf_data` - The ELF binary data for the application.
    /// * `app_id` - The application identifier (used for kernel stack allocation).
    ///
    /// # Returns
    /// A fully initialized `TaskControlBlock` ready to be scheduled.
    pub fn new(app_id: usize, elf_data: &[u8]) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT_ADDR).floor())
            .unwrap()
            .ppn();
        let task_status = TaskStatus::Ready;

        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_pos(app_id);
        KERNEL_SPACE.exclusive_access().insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );
        let task_control_block = Self {
            task_status,
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            memory_set,
            trap_cx_ppn,
            base_size: user_sp.bits(),
        };

        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::init_context(
            entry_point,
            user_sp.bits(),
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }

    /// Returns a mutable reference to the trap context for this task.
    ///
    /// The trap context holds the processor state to be restored when returning
    /// from a trap (interrupt, exception, or syscall).
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    /// Returns the SATP value for this task's address space.
    ///
    /// This value encodes the page table root and mode for address translation,
    /// and is used to activate the task's memory mapping.
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
}
