use super::context::TaskContext;
use crate::config::{self, TRAP_CONTEXT_ADDR};
use crate::mm::address::{PhysPageNum, VirtAddr};
use crate::mm::memory_set::{KERNEL_SPACE, MapPermission, MemorySet};
use crate::trap::context::TrapContext;
use crate::trap::trap_handler;

/// The TaskControlBlock holds all information needed to manage and schedule a task.
///
/// Fields:
/// - `task_status`: The current status of the task (e.g., Ready, Running, Exited).
/// - `task_ctx`: The saved CPU context for context switching.
/// - `memory_set`: The address space and memory mappings for the task.
/// - `trap_ctx_ppn`: The physical page number of the trap context for this task.
/// - `base_size`: The size of the application from address 0x0 to the top of the user stack.
pub struct TaskControlBlock {
    pub task_ctx: TaskContext,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,
    pub trap_ctx_ppn: PhysPageNum,
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
        // Allocate kernel stack for the app
        let (kstack_bottom, kstack_top) = config::kernel_stack_pos(app_id);
        KERNEL_SPACE.exclusive_access().insert_framed_area(
            kstack_bottom.into(),
            kstack_top.into(),
            MapPermission::R | MapPermission::W,
        );

        // Load ELF and set up user address space
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_ctx_ppn = memory_set
            .page_table
            .translate(VirtAddr::from(TRAP_CONTEXT_ADDR).floor())
            .expect("Failed to translate TRAP_CONTEXT_ADDR")
            .ppn();

        // Construct TaskControlBlock
        let tcb = TaskControlBlock {
            task_ctx: TaskContext::goto_trap_return(kstack_top),
            task_status: TaskStatus::Ready,
            memory_set,
            trap_ctx_ppn,
            base_size: user_sp.bits, // from 0x0 to stack top
        };

        // Initialize TrapContext in user space
        let trap_ctx = tcb.get_trap_ctx_mut();
        *trap_ctx = TrapContext::init_ctx(
            entry_point,
            user_sp.bits,
            KERNEL_SPACE.exclusive_access().satp(),
            kstack_top,
            trap_handler as usize,
        );

        tcb
    }

    /// Returns the SATP value for this task's address space.
    ///
    /// This value encodes the page table root and mode for address translation,
    /// and is used to activate the task's memory mapping.
    pub fn satp(&self) -> usize {
        self.memory_set.satp()
    }

    /// Returns a mutable reference to the trap context for this task.
    ///
    /// The trap context holds the processor state to be restored when returning
    /// from a trap (interrupt, exception, or syscall).
    pub fn get_trap_ctx_mut(&self) -> &'static mut TrapContext {
        self.trap_ctx_ppn.get_mut()
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}
