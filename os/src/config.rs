/// User stack size in bytes (8 KiB).
pub const USER_STACK_SIZE: usize = 4096 * 2;

/// Kernel stack size in bytes (8 KiB).
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;

/// Kernel heap size in bytes (3 MiB).
pub const KERNEL_HEAP_SIZE: usize = 3 * 1024 * 1024; // 0x30_0000

/// Page offset bits for SV39
pub const PAGE_OFFSET_BITS: usize = 12;

/// Page size in bytes (4 KiB).
pub const PAGE_SIZE: usize = 1 << PAGE_OFFSET_BITS;

/// Address of the trampoline code (top of virtual address space).
///
/// This address is set to the highest possible value in the virtual address space (`usize::MAX - PAGE_SIZE + 1`).
/// In the SV39 scheme, the virtual address space is 39 bits (0x0 ~ 0x7FFF_FFFF_FFFF),
/// while `usize::MAX` on a 64-bit system is 0xFFFF_FFFF_FFFF_FFFF.
/// Although the calculated `TRAMPOLINE_ADDR` appears to exceed the SV39 range,
/// the hardware only uses the lower 39 bits (with sign extension),
/// so this address is mapped to the highest region of the SV39 virtual address space (0xFFFF_FFFF_FFFF).
pub const TRAMPOLINE_ADDR: usize = usize::MAX - PAGE_SIZE + 1;

/// Address for the trap context (just below the trampoline).
pub const TRAP_CONTEXT_ADDR: usize = TRAMPOLINE_ADDR - PAGE_SIZE;

/// Returns the bottom and top addresses of the kernel stack for a given app.
///
/// # Arguments
///
/// * `app_id` - The application identifier (used to calculate stack position).
///
/// # Returns
///
/// A tuple `(bottom, top)` representing the stack's address range.
pub fn kernel_stack_pos(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE_ADDR - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
