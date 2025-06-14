use riscv::register::sstatus::{self, SPP, Sstatus};

#[repr(C)]
/// The trap context structure used to save and restore processor state during a trap (interrupt, exception, or syscall).
///
/// `TrapContext` holds all general-purpose registers, status, and control information needed to resume execution
/// after handling a trap. It is used to switch between user and kernel mode safely.
///
/// Fields:
/// - `x`: General-purpose registers x0-x31 (x[0] is unused, x[1]-x[31] are saved/restored)
/// - `sstatus`: The supervisor status register, storing privilege and interrupt state
/// - `sepc`: The supervisor exception program counter, storing the return address for sret
/// - `kernel_satp`: The kernel page table root (SATP register value)
/// - `kernel_sp`: The kernel stack pointer for trap handling
/// - `trap_handler`: The address of the kernel's trap handler function
pub struct TrapContext {
    /// general regs[0..31]
    pub x: [usize; 32],
    /// CSR sstatus
    pub sstatus: Sstatus,
    /// CSR sepc
    pub sepc: usize,
    /// Addr of Page Table
    pub kernel_satp: usize,
    /// kernel stack
    pub kernel_sp: usize,
    /// Addr of trap_handler function
    pub trap_handler: usize,
}

impl TrapContext {
    /// set stack pointer to x_2 reg (sp)
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    /// Initialize a new trap context for entering user mode.
    ///
    /// This function sets up a `TrapContext` with the specified entry point, user stack pointer,
    /// kernel SATP (page table), kernel stack pointer, and trap handler address.
    /// It configures the status register to return to user mode on `sret`.
    ///
    /// # Arguments
    /// * `entry` - The entry point address for user code (to be set in `sepc`).
    /// * `sp` - The user stack pointer value.
    /// * `kernel_satp` - The SATP value for the kernel address space.
    /// * `kernel_sp` - The kernel stack pointer for trap handling.
    /// * `trap_handler` - The address of the kernel trap handler function.
    ///
    /// # Returns
    /// A fully initialized `TrapContext` ready for user mode execution.
    pub fn init_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut sstatus = sstatus::read(); // CSR sstatus
        sstatus.set_spp(SPP::User); //previous privilege mode: user mode
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,  // entry point of app
            kernel_satp,  // addr of page table
            kernel_sp,    // kernel stack
            trap_handler, // addr of trap_handler function
        };
        cx.set_sp(sp); // app's user stack pointer
        cx // return initial Trap Context of app
    }
}
