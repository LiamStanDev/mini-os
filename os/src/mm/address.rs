use super::page_table::PageTableEntry;
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

/// Represents a physical address.
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct PhysAddr {
    /// The address bits.
    pub bits: usize,
}

/// Represents a virtual address.
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct VirtAddr {
    /// The address bits.
    pub bits: usize,
}

/// Represents a physical page number.
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct PhysPageNum {
    /// The page number bits.
    pub bits: usize,
}

/// Represents a virtual page number.
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct VirtPageNum {
    /// The page number bits.
    pub bits: usize,
}

impl PhysAddr {
    pub fn page_offset(&self) -> usize {
        self.bits & (PAGE_SIZE - 1)
    }

    /// Returns the physical page number containing this address (rounded down).
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum {
            bits: self.bits / PAGE_SIZE,
        }
    }

    /// Returns the physical page number containing this address (rounded up).
    pub fn ceil(&self) -> PhysPageNum {
        if (self.bits == 0) {
            PhysPageNum { bits: 0 }
        } else {
            #[allow(clippy::manual_div_ceil)]
            PhysPageNum {
                bits: (self.bits + PAGE_SIZE - 1) / PAGE_SIZE,
            }
        }
    }
}

impl PhysPageNum {
    /// Returns the starting physical address of this page.
    pub fn get_addr0(&self) -> PhysAddr {
        PhysAddr {
            bits: self.bits << PAGE_OFFSET_BITS,
        }
    }

    /// Returns a mutable byte slice representing the page's memory.
    pub fn get_bytes_array_mut(&self) -> &'static mut [u8] {
        let pa: PhysAddr = self.get_addr0();
        unsafe { core::slice::from_raw_parts_mut(pa.bits as *mut u8, PAGE_SIZE) }
    }

    /// Returns an immutable byte slice representing the page's memory.
    pub fn get_bytes_array(&self) -> &'static [u8] {
        let pa: PhysAddr = self.get_addr0();
        unsafe { core::slice::from_raw_parts(pa.bits as *mut u8, PAGE_SIZE) }
    }

    /// Returns a mutable slice of page table entries for this page.
    ///
    /// # Safety
    /// The page must be used as a page table.
    pub fn get_pte_array_mut(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = self.get_addr0();
        // PAGE_SIZE / sizeof(PageTableEntry) = 512; one page can store 512 PTEs.
        unsafe { core::slice::from_raw_parts_mut(pa.bits as *mut PageTableEntry, PAGE_SIZE / 8) }
    }

    /// Returns a mutable reference to a value of type `T` at the start of this page.
    ///
    /// # Safety
    /// The caller must ensure the type and alignment are correct.
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).get_addr0();
        unsafe { (pa.bits as *mut T).as_mut().unwrap() }
    }
}

impl VirtAddr {
    /// Returns the offset within the page for this virtual address.
    pub fn page_offset(&self) -> usize {
        self.bits & (PAGE_SIZE - 1)
    }

    /// Returns the virtual page number containing this address (rounded down).
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum {
            bits: self.bits / PAGE_SIZE,
        }
    }

    /// Returns the virtual page number containing this address (rounded up).
    pub fn ceil(&self) -> VirtPageNum {
        VirtPageNum {
            bits: (self.bits + PAGE_SIZE - 1) / PAGE_SIZE,
        }
    }
}

impl VirtPageNum {
    /// Returns the starting virtual address of this page.
    pub fn get_addr0(&self) -> VirtAddr {
        VirtAddr {
            bits: self.bits << PAGE_OFFSET_BITS,
        }
    }

    /// Returns the SV39 page table indexes for this virtual page number.
    ///
    /// The result is an array of 3 indexes, one for each level of the page table.
    pub fn get_sv39_indexes(&self) -> [usize; 3] {
        const MASK: usize = 0b1_1111_1111;
        [
            (self.bits >> 0) & MASK,
            (self.bits >> 9) & MASK,
            (self.bits >> 18) & MASK,
        ]
    }
}

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.bits))
    }
}
impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VPN:{:#x}", self.bits))
    }
}
impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.bits))
    }
}
impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PPN:{:#x}", self.bits))
    }
}

// from usize

impl From<usize> for PhysAddr {
    /// Converts a `usize` to a `PhysAddr`, masking to the lower 56 bits.
    fn from(v: usize) -> Self {
        Self {
            // lower 56 bits
            bits: v & ((1 << PA_WIDTH_SV39) - 1),
        }
    }
}

impl From<usize> for PhysPageNum {
    /// Converts a `usize` to a `PhysPageNum`, masking to the lower 44 bits.
    fn from(v: usize) -> Self {
        Self {
            // lower 44 bits
            bits: v & ((1 << PPN_WIDTH_SV39) - 1),
        }
    }
}

impl From<usize> for VirtAddr {
    /// Converts a `usize` to a `VirtAddr`, masking to the lower 39 bits.
    fn from(v: usize) -> Self {
        Self {
            // lower 39 bits
            bits: v & ((1 << VA_WIDTH_SV39) - 1),
        }
    }
}
impl From<usize> for VirtPageNum {
    /// Converts a `usize` to a `VirtPageNum`, masking to the lower 27 bits.
    fn from(v: usize) -> Self {
        Self {
            // lower 27 bits
            bits: v & ((1 << VPN_WIDTH_SV39) - 1),
        }
    }
}
