/// Frame allocator module for managing physical memory frames.
use super::address::PhysPageNum;
use crate::board::MEMORY_END;
use crate::config::PAGE_SIZE;
use crate::mm::address::PhysAddr;
use crate::sync::UPSafeCell;
use crate::*;
use alloc::vec::Vec;
use core::fmt::{self, Debug, Formatter};
use lazy_static::*;

/// Initialize the global frame allocator.
///
/// This function sets up the frame allocator to manage all physical memory frames
/// between the end of the kernel and the end of physical memory.
pub fn init_frame_allocator() {
    let start_pa = PhysAddr::from(ekernel as usize);
    let end_pa = PhysAddr::from(MEMORY_END);

    // NOTE: frame allocator init by PPN not PA
    // Use ceil/floor to convert addr to corresponding page number
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as usize).ceil(),
        PhysAddr::from(MEMORY_END).floor(),
    );
}

/// Allocate a physical frame and return a `FrameTracker` if successful.
///
/// # Returns
/// - `Some(FrameTracker)` if a frame is available.
/// - `None` if no frames are available.
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(FrameTracker::new)
}

/// Deallocate a physical frame given its page number.
///
/// # Arguments
/// - `ppn`: The physical page number to deallocate.
pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

/// Tracks the allocation of a physical frame.
///
/// When dropped, the frame is automatically deallocated.
pub struct FrameTracker {
    /// The physical page number of the tracked frame.
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    /// Create a new `FrameTracker` for the given physical page number.
    ///
    /// The frame's memory is zeroed on allocation.
    pub fn new(ppn: PhysPageNum) -> Self {
        let bytes_array = ppn.get_bytes_array();
        bytes_array.fill(0);
        Self { ppn }
    }
}

impl Drop for FrameTracker {
    /// Automatically deallocate the frame when the tracker is dropped.
    fn drop(&mut self) {
        frame_dealloc(self.ppn)
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

lazy_static! {
    /// Global frame allocator instance, protected by a lock.
    static ref FRAME_ALLOCATOR: UPSafeCell<StackFrameAllocator> =
        unsafe { UPSafeCell::new(StackFrameAllocator::new()) };
}

/// Trait for frame allocator implementations.
pub trait FrameAllocator {
    /// Create a new frame allocator instance.
    fn new() -> Self;
    /// Allocate a physical page number.
    fn alloc(&mut self) -> Option<PhysPageNum>;
    /// Deallocate a physical page number.
    fn dealloc(&mut self, ppn: PhysPageNum);
}

/// Stack-based frame allocator implementation.
pub struct StackFrameAllocator {
    /// Next free physical page number.
    current: usize,
    /// End of the managed physical page range (exclusive).
    end: usize,
    /// Stack of recycled (freed) physical page numbers.
    recycled: Vec<usize>, // store recycled ppn
}

impl StackFrameAllocator {
    /// Initialize the allocator with a range of physical page numbers.
    ///
    /// # Arguments
    /// - `start`: The first physical page number to manage.
    /// - `end`: The last physical page number to manage (exclusive).
    pub fn init(&mut self, start: PhysPageNum, end: PhysPageNum) {
        self.current = start.0;
        self.end = end.0;
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end {
            log::warn!(
                "Frame allocator out of memory! current={:#x}, end={:#x}",
                self.current,
                self.end
            );
            None
        } else {
            let ppn = self.current;
            self.current += 1;
            Some((self.current - 1).into())
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        if ppn >= self.current || self.recycled.contains(&ppn) {
            panic!("Frame ppn={:#x} has not been has not been allocated!", ppn);
        }

        // recycle
        self.recycled.push(ppn);
    }
}

#[allow(unused)]
/// Test function for the frame allocator.
///
/// Allocates and deallocates several frames to verify correct behavior.
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}
