use super::PageTableEntry;
use crate::config::{PAGE_OFFSET_BITS, PAGE_SIZE};
use core::fmt::{self, Debug, Formatter};

/// Physical address width for SV39 (in bits)
const PA_WIDTH_SV39: usize = 56;
/// Virtual address width for SV39 (in bits)
const VA_WIDTH_SV39: usize = 39;
/// Physical page number width for SV39 (in bits)
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_OFFSET_BITS;
/// Virtual page number width for SV39 (in bits)
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_OFFSET_BITS;

/// Definition

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

/// Debugging
impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:#x}(PA)", self.0))
    }
}
impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:#x}(VA)", self.0))
    }
}
impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:#x}(PPN)", self.0))
    }
}
impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:#x}(VPN)", self.0))
    }
}

/// usize <-> T
impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << PA_WIDTH_SV39) - 1))
    }
}
impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VA_WIDTH_SV39) - 1))
    }
}
impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << PPN_WIDTH_SV39) - 1))
    }
}
impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VPN_WIDTH_SV39) - 1))
    }
}

impl VirtAddr {
    pub fn bits(&self) -> usize {
        self.0
    }

    /// Returns the offset within the page for this virtual address.
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }

    /// Returns the virtual page number containing this address (rounded down).
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }

    /// Returns the virtual page number containing this address (rounded up).
    pub fn ceil(&self) -> VirtPageNum {
        VirtPageNum(self.0.div_ceil(PAGE_SIZE))
    }

    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

impl VirtPageNum {
    pub fn bits(&self) -> usize {
        self.0
    }

    pub fn add(&mut self, v: usize) {
        self.0 += v;
    }

    /// Returns the SV39 page table indexes for this virtual page number.
    ///
    /// The result is an array of 3 indexes, one for each level of the page table.
    pub fn indexes(&self) -> [usize; 3] {
        const MASK: usize = 0b1_1111_1111;
        [
            (self.0 >> 18) & MASK,
            (self.0 >> 9) & MASK,
            (self.0 >> 0) & MASK,
        ]
    }

    /// Returns the starting virtual address of this page.
    pub fn get_first_addr(&self) -> VirtAddr {
        VirtAddr(self.0 << PAGE_OFFSET_BITS)
    }
}

impl PhysAddr {
    pub fn bits(&self) -> usize {
        self.0
    }

    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }

    /// Returns the physical page number containing this address (rounded down).
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }

    /// Returns the physical page number containing this address (rounded up).
    pub fn ceil(&self) -> PhysPageNum {
        PhysPageNum(self.0.div_ceil(PAGE_SIZE))
    }

    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

impl PhysPageNum {
    pub fn bits(&self) -> usize {
        self.0
    }

    pub fn add(&mut self, v: usize) {
        self.0 += v;
    }

    /// Returns the starting physical address of this page.
    pub fn get_first_addr(&self) -> PhysAddr {
        PhysAddr(self.0 << PAGE_OFFSET_BITS)
    }

    /// Returns a mutable byte slice representing the page's memory.
    pub fn get_bytes_array_mut(&self) -> &'static mut [u8] {
        let pa: PhysAddr = self.get_first_addr();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, PAGE_SIZE) }
    }

    /// Returns an immutable byte slice representing the page's memory.
    pub fn get_bytes_array(&self) -> &'static [u8] {
        let pa: PhysAddr = self.get_first_addr();
        unsafe { core::slice::from_raw_parts(pa.0 as *mut u8, PAGE_SIZE) }
    }

    /// Returns a mutable slice of page table entries for this page.
    ///
    /// # Safety
    /// The page must be used as a page table.
    pub fn get_pte_array_mut(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = self.get_first_addr();
        // PAGE_SIZE / sizeof(PageTableEntry) = 512; one page can store 512 PTEs.
        unsafe {
            core::slice::from_raw_parts_mut(
                pa.0 as *mut PageTableEntry,
                PAGE_SIZE / size_of::<PageTableEntry>(),
            )
        }
    }

    /// Returns a mutable reference to a value of type `T` at the start of this page.
    ///
    /// # Safety
    /// The caller must ensure the type and alignment are correct.
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).get_first_addr();
        unsafe { (pa.0 as *mut T).as_mut().unwrap() }
    }
}
