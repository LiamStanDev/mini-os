use log::*;

use self::memory_set::KERNEL_SPACE;

pub mod address;
mod frame_allocator;
mod heap_allocator;
pub mod memory_set;
mod page_table;

pub fn init() {
    // Initializes the heap allocator for dynamic memory allocation.
    heap_allocator::init_heap();
    // heap_allocator::heap_test();

    // Initializes the physical frame allocator for managing physical memory pages.
    frame_allocator::init_frame_allocator();
    // frame_allocator::frame_allocator_test();

    // Activates the kernel's page table and enables paging mode.
    // memory_set::activate_kernel();

    KERNEL_SPACE.exclusive_access().activate();
    // memory_set::remap_kernel_test();
}
