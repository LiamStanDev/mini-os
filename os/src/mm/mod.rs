//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.

pub mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use memory_set::{KERNEL_SPACE, MapPermission, MemorySet};
pub use page_table::PageTableEntry;

use self::frame_allocator::frame_allocator_test;
use self::heap_allocator::heap_test;
use self::memory_set::activate_kernel;

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::init_heap();
    heap_test();
    frame_allocator::init_frame_allocator();
    frame_allocator_test();
    activate_kernel();
}
