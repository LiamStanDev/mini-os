use super::address::{PhysPageNum, VirtAddr, VirtPageNum};
use super::frame_allocator::{FrameTracker, frame_alloc};
use crate::config::PAGE_SIZE;
use alloc::vec;
use alloc::vec::Vec;
use bitflags::bitflags;

/// Page table structure for virtual memory management.
///
/// Note: One PageTable per application (kernel/user)
///
/// Each `PageTable` instance manages the root physical page number and a list of
/// allocated frames for all page table nodes. Typically, one page table is created
/// per address space (kernel or user).
pub struct PageTable {
    /// The root physical page number of the page table.
    pub root_ppn: PhysPageNum,
    /// Frames allocated for page table nodes, to be deallocated when dropped.
    frames: Vec<FrameTracker>,
}

impl PageTable {
    /// Create a new page table with a freshly allocated root frame.
    ///
    /// Allocates a new frame for the root page table node and tracks it for later deallocation.
    pub fn new() -> Self {
        let frame = frame_alloc().expect("frame_alloc failed!");
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }

    /// Construct a page table from an existing SATP value (root physical page number).
    ///
    /// This does not take ownership of any frames and is typically used for temporary access.
    pub fn from_satp(satp: usize) -> Self {
        Self {
            root_ppn: satp.into(),
            frames: vec![],
        }
    }

    /// Translate a virtual page number to its corresponding page table entry, if mapped.
    ///
    /// # Arguments
    /// * `vpn` - The virtual page number to translate.
    ///
    /// # Returns
    /// * `Some(PageTableEntry)` if the mapping exists.
    /// * `None` if the mapping does not exist.
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte_mut(vpn).map(|pte| *pte) // NOTE: PageTableEntry is Copy trait
    }

    /// Translate a virtual address range into a vector of byte slices mapped in physical memory.
    ///
    /// This function walks the page table and collects all contiguous physical memory slices
    /// that correspond to the given virtual address range `[ptr, ptr + len)`. The result is a
    /// vector of references to the mapped physical memory regions, which may span multiple pages.
    ///
    /// # Arguments
    /// * `satp` - The SATP value representing the root page table.
    /// * `ptr` - The starting virtual address as a raw pointer.
    /// * `len` - The length in bytes of the virtual memory region to translate.
    ///
    /// # Returns
    /// A vector of byte slices (`&'static [u8]`), each representing a contiguous region of
    /// mapped physical memory corresponding to the requested virtual address range.
    ///
    /// # Panics
    /// Panics if any part of the virtual address range cannot be translated.
    pub fn translated_byte_buffer(satp: usize, ptr: *const u8, len: usize) -> Vec<&'static [u8]> {
        let page_table = PageTable::from_satp(satp); // get non-owned PageTable from satp
        let start_addr = ptr as usize;
        let end_addr = start_addr + len;
        let mut res = Vec::new();

        let mut current = start_addr;
        while current < end_addr {
            let va = VirtAddr::from(start_addr);
            let vpn = va.floor();
            let ppn = page_table
                .translate(vpn)
                .expect("cannot translate page")
                .ppn();

            let page_start = vpn.into();
            let page_end = page_start + PAGE_SIZE;

            #[rustfmt::skip]
            let slice_start = current.saturating_sub(page_start); // >= 0
            #[rustfmt::skip]
            let slice_end = if end_addr < page_end { end_addr - page_start } else { PAGE_SIZE };

            res.push(&ppn.get_bytes_array()[slice_start..slice_end]);
            current = page_start + slice_end;
        }

        res
    }

    /// Map a virtual page number to a physical page number with the given flags.
    ///
    /// # Arguments
    /// * `vpn` - The virtual page number to map.
    /// * `ppn` - The physical page number to map to.
    /// * `flags` - The page table entry flags.
    ///
    /// # Panics
    /// Panics if the virtual page is already mapped.
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self
            .find_pte_create_mut(vpn)
            .expect("call find_pte_create_mut to map vpn {vpn:?}");
        assert!(!pte.is_valid(), "vpn {vpn:?} is mapped before mapping");
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    /// Unmap a virtual page number.
    ///
    /// # Arguments
    /// * `vpn` - The virtual page number to unmap.
    ///
    /// # Panics
    /// Panics if the virtual page is not mapped.
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self
            .find_pte_mut(vpn)
            .expect("call ummap to unmaped vpn {vpn:?} unmaped");
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }

    /// Find a mutable reference to the page table entry for the given virtual page number.
    ///
    /// Returns `None` if any intermediate page table is missing or invalid.
    fn find_pte_mut(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, &idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[idx];
            if i == 2 {
                // last page table
                result = Some(pte);
                break;
            }

            if !pte.is_valid() {
                return None;
            }

            ppn = pte.ppn(); // move to next table
        }

        result
    }

    /// Find or create the page table entry for the given virtual page number.
    ///
    /// If any intermediate page table is missing, it will be allocated and tracked.
    fn find_pte_create_mut(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result = None;

        for (i, &idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[idx];
            if i == 2 {
                // last page table
                result = Some(pte);
                break;
            }

            // create page table
            if !pte.is_valid() {
                let frame = frame_alloc().unwrap_or_else(|| {
                    panic!(
                        "frame alloc failed when create PTE for mapping {:?} in level-{} page table",
                        vpn,
                        i + 1
                    )
                });

                // NOTE: V is 1 and R/W/X all 0 means this page is a valid page table
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }

            ppn = pte.ppn();
        }

        result
    }
}

/// A page table entry (PTE) in the SV39 page table format.
///
/// Each entry contains the physical page number and permission/status flags.
/// The entry is represented as a 64-bit value (8 bytes).
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PageTableEntry {
    /// The raw bits of the page table entry (8 bytes).
    pub bits: usize,
}

impl PageTableEntry {
    /// Creates a new page table entry with the given physical page number and flags.
    ///
    /// # Arguments
    /// * `ppn` - The physical page number to store in the entry.
    /// * `flags` - The permission and status flags for the entry.
    ///
    /// # Returns
    /// A new `PageTableEntry` with the specified physical page number and flags.
    fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits() as usize,
        }
    }

    /// Returns an empty (invalid) page table entry.
    fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }

    /// Returns the physical page number stored in this entry.
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1 << 44) - 1)).into()
    }

    /// Returns the flags stored in this entry.
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    /// Returns `true` if the entry is valid.
    pub fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::V)
    }

    /// Returns `true` if the entry is readable.
    pub fn readable(&self) -> bool {
        self.flags().contains(PTEFlags::R)
    }

    /// Returns `true` if the entry is writable.
    pub fn writable(&self) -> bool {
        self.flags().contains(PTEFlags::W)
    }

    /// Returns `true` if the entry is executable.
    pub fn executable(&self) -> bool {
        self.flags().contains(PTEFlags::X)
    }
}

bitflags! {
    /// Page table entry flags for SV39 page tables.
    ///
    /// Each flag represents a permission or status bit in the PTE.
    /// Multiple flags can be combined using bitwise OR.
    pub struct PTEFlags: u8 {
        /// Valid
        const V = 1 << 0;
        /// Readable
        const R = 1 << 1;
        /// Writable
        const W = 1 << 2;
        /// Executable
        const X = 1 << 3;
        /// User
        const U = 1 << 4;
        /// Global
        const G = 1 << 5;
        /// Accessed
        const A = 1 << 6;
        /// Dirty
        const D = 1 << 7;
    }
}
