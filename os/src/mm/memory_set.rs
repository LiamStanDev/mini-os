use super::PageTableEntry;
use super::address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use super::frame_allocator::{FrameTracker, frame_alloc};
use super::page_table::{PTEFlags, PageTable};
use crate::board::MEMORY_END;
use crate::config::{PAGE_SIZE, TRAMPOLINE_ADDR, TRAP_CONTEXT_ADDR, USER_STACK_SIZE};
use crate::sync::*;
use crate::*;
use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use bitflags::bitflags;
use core::arch::asm;
use lazy_static::lazy_static;
use log::*;
use riscv::register;
use riscv::register::satp::Satp;

/// Activates the kernel's address space by loading its page table into the hardware.
///
/// This function sets the current address space to the kernel space by activating
/// the kernel's root page table.
pub fn activate_kernel() {
    KERNEL_SPACE.exclusive_access().activate();
}

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> =
        Arc::new(unsafe { UPSafeCell::new(MemorySet::init_kernel_space()) });
}

/// MemorySet represents the address space of a process.
///
/// Each process has its own `MemorySet`, which contains the page table and all mapped memory areas.
pub struct MemorySet {
    /// The page table for this address space.
    pub page_table: PageTable,
    /// All memory areas mapped in this address space.
    areas: Vec<MapArea>,
}

impl MemorySet {
    /// Create a new, empty MemorySet with an empty page table and no mapped areas.
    pub fn default() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    /// Add a new memory area to the address space and optionally initialize its contents.
    ///
    /// # Arguments
    /// * `map_area` - The memory area to map.
    /// * `bytes` - Optional byte slice to initialize the mapped area.
    fn push(&mut self, mut map_area: MapArea, bytes: Option<&[u8]>) {
        map_area.map(&mut self.page_table);

        if let Some(bytes) = bytes {
            map_area.write_bytes(&mut self.page_table, bytes);
        }

        self.areas.push(map_area);
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }

    /// Insert a new framed memory area into the address space.
    ///
    /// # Arguments
    /// * `start_va` - Start virtual address of the area.
    /// * `end_va` - End virtual address of the area.
    /// * `permission` - Permissions for the mapped area.
    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        self.push(
            MapArea::new(start_va, end_va, MapType::Framed, permission),
            None,
        );
    }

    /// Create a new `MemorySet` for the kernel address space.
    ///
    /// This function constructs a `MemorySet` and maps all necessary kernel sections,
    /// including .text, .rodata, .data, .bss, and the remaining physical memory.
    /// All mappings use identical mapping (virtual address equals physical address)
    /// and do not grant user permissions for safety.
    ///
    /// # Returns
    /// A fully initialized `MemorySet` representing the kernel address space.
    pub fn init_kernel_space() -> Self {
        let mut memory_set = Self::default();

        // map trampoline
        memory_set.map_trampoline();

        let sections = [
            (
                (stext as usize, etext as usize),
                MapPermission::R | MapPermission::X,
                ".text",
            ),
            (
                (srodata as usize, erodata as usize),
                MapPermission::R,
                ".rodata",
            ),
            (
                (sdata as usize, edata as usize),
                MapPermission::R | MapPermission::W,
                ".data",
            ),
            (
                (sbss_with_stack as usize, ebss as usize),
                MapPermission::R | MapPermission::W,
                ".bss",
            ),
            (
                (ekernel as usize, MEMORY_END),
                MapPermission::R | MapPermission::W,
                "physical memory",
            ),
        ];

        for &((start, end), perm, name) in &sections {
            trace!("mapping {name} section [{start:#x}, {end:#x})");
            memory_set.push(
                MapArea::new(start.into(), end.into(), MapType::Identical, perm),
                None,
            );
        }

        memory_set
    }

    /// Create a new `MemorySet` from an ELF binary.
    ///
    /// This function parses the ELF file, maps all loadable segments into the address space,
    /// sets up the user stack with a guard page, reserves space for `sbrk`, and maps the trap context.
    ///
    /// # Arguments
    /// * `elf_data` - The ELF binary data as a byte slice.
    ///
    /// # Returns
    /// A tuple containing:
    /// - The constructed `MemorySet`
    /// - The top of the user stack (`VirtAddr`)
    /// - The entry point address (`usize`)
    pub fn from_elf(elf_data: &[u8]) -> (Self, VirtAddr, usize) {
        let mut memory_set = Self::default();

        memory_set.map_trampoline();

        let elf = xmas_elf::ElfFile::new(elf_data).expect("failed to parse elf data");
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count(); // program header count

        let mut max_end_vpn: VirtPageNum = (0usize).into();

        fn elf_segment_perm(ph_flags: xmas_elf::program::Flags) -> MapPermission {
            let mut perm = MapPermission::U;
            if ph_flags.is_read() {
                perm |= MapPermission::R;
            }
            if ph_flags.is_write() {
                perm |= MapPermission::W;
            }
            if ph_flags.is_execute() {
                perm |= MapPermission::X;
            }
            perm
        }

        (0..ph_count)
            .filter_map(|i| {
                let ph = elf.program_header(i).ok()?;
                if ph.get_type().ok()? != xmas_elf::program::Type::Load {
                    return None;
                }
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let perm = elf_segment_perm(ph.flags());
                // Note: mem_size >= file_size (only code and data)
                let file_range = ph.offset() as usize..(ph.offset() + ph.file_size()) as usize;
                Some((start_va, end_va, perm, &elf.input[file_range]))
            })
            .for_each(|(start_va, end_va, perm, data)| {
                let map_area = MapArea::new(start_va, end_va, MapType::Framed, perm);
                max_end_vpn = map_area.vpn_range.end;
                memory_set.push(map_area, Some(data));
            });

        // stack
        let mut user_stack_bottom: VirtAddr = max_end_vpn.get_first_addr();
        user_stack_bottom.0 += PAGE_SIZE; // guard page
        let user_stack_top: VirtAddr = (user_stack_bottom.0 + USER_STACK_SIZE).into();
        memory_set.push(
            MapArea::new(
                user_stack_bottom,
                user_stack_top,
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );

        // map TrapContext
        memory_set.push(
            MapArea::new(
                VirtAddr::from(TRAP_CONTEXT_ADDR),
                VirtAddr::from(TRAMPOLINE_ADDR),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        (
            memory_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }

    /// returns the value that should be written to the RISC-V satp
    pub fn token(&self) -> usize {
        let mut satp = register::satp::read();
        satp.set_mode(register::satp::Mode::Sv39);
        satp.set_ppn(self.page_table.root_ppn.0);

        satp.bits()
    }

    /// Activate this address space by loading its page table into the hardware.
    ///
    /// This function sets the SATP register to the root page table of this `MemorySet`
    /// and flushes the TLB to ensure address translation uses the new mappings.
    pub fn activate(&self) {
        let satp = Satp::from_bits(self.token());

        unsafe {
            register::satp::write(satp);
            // flush TLB
            asm!("sfence.vma");
        }
    }

    /// Map the trampoline code into the address space.
    ///
    /// This function maps the trampoline virtual address to the physical address
    /// of the trampoline code(specified in linker) with read and execute permissions.
    /// The trampoline is used for context switching and trap handling.
    fn map_trampoline(&mut self) {
        let vpn = VirtAddr::from(TRAMPOLINE_ADDR).floor();
        let ppn = PhysAddr::from(strampoline as usize).floor();
        trace!("mapping trampoline: {vpn:#?} -> {ppn:#?}");
        self.page_table.map(vpn, ppn, PTEFlags::R | PTEFlags::X);
    }
}

/// Describes a continuous range of virtual pages with the same mapping type and permissions.
///
/// `MapArea` manages the mapping between a range of virtual page numbers and their corresponding
/// physical frames, along with the mapping type and permissions for that region.
pub struct MapArea {
    /// The range of virtual page numbers covered by this area.
    vpn_range: VPNRange,
    /// Mapping from virtual page numbers to their allocated physical frames.
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    /// The type of mapping (e.g., Identical, Framed).
    map_type: MapType,
    /// The permissions for this memory area.
    map_perm: MapPermission,
}

impl MapArea {
    /// Create a new `MapArea` for a given virtual address range, mapping type, and permissions.
    ///
    /// Note: This method only constructs the `MapArea` data structure. It does **not**
    /// create or initialize any mappings in the page table yet. The actual mapping
    /// in the page table will be performed when calling `map()`.
    ///
    /// # Arguments
    /// * `start_va` - The start virtual address (inclusive).
    /// * `end_va` - The end virtual address (exclusive).
    /// * `map_type` - The type of mapping (e.g., Identical, Framed).
    /// * `map_perm` - The permissions for this memory area.
    ///
    /// # Returns
    /// A new `MapArea` covering the specified virtual address range.
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        let start: VirtPageNum = start_va.floor();
        let end: VirtPageNum = end_va.ceil();
        Self {
            vpn_range: VPNRange::new(start, end),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }

    /// Map all virtual pages in the area using the provided page table.
    ///
    /// Calls `map_one` for each virtual page number in the range.
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }

    /// Unmap all virtual pages in the area using the provided page table.
    ///
    /// Calls `unmap_one` for each virtual page number in the range.
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn);
        }
    }

    /// Map a single virtual page in this area using the provided page table.
    ///
    /// Allocates a physical frame if the mapping type is `Framed`, or uses the same page number
    /// for `Identical` mapping. Updates the page table with the mapping and permissions.
    ///
    /// # Arguments
    /// * `page_table` - The page table to update.
    /// * `vpn` - The virtual page number to map.
    fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn: PhysPageNum = match self.map_type {
            MapType::Identical => vpn.0.into(),
            MapType::Framed => {
                let frame = frame_alloc().expect("failed to alloc frame when using map_one");
                let ppn = frame.ppn;
                self.data_frames.insert(vpn, frame);
                ppn
            }
        };

        let pte_flags =
            PTEFlags::from_bits(self.map_perm.bits()).expect("invalid MapPermission bits");
        page_table.map(vpn, ppn, pte_flags);
    }

    /// Unmap a single virtual page in this area using the provided page table.
    ///
    /// Removes the frame from `data_frames` if the mapping type is `Framed`, and updates the page table.
    ///
    /// # Arguments
    /// * `page_table` - The page table to update.
    /// * `vpn` - The virtual page number to unmap.
    fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        match self.map_type {
            MapType::Framed => {
                self.data_frames.remove(&vpn);
            }
            _ => {}
        }

        page_table.unmap(vpn);
    }

    /// Write a byte slice into the mapped memory area using the provided page table.
    ///
    /// This method copies the contents of `bytes` into the physical memory frames
    /// mapped by this `MapArea`. The area must use `MapType::Framed`.
    ///
    /// # Arguments
    /// * `page_table` - The page table used for address translation.
    /// * `bytes` - The byte slice to write into the mapped memory area.
    ///
    /// # Panics
    /// Panics if the mapping type is not `Framed` or if translation fails.
    pub fn write_bytes(&mut self, page_table: &mut PageTable, bytes: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);

        let mut vpn_iter = self.vpn_range.into_iter();

        for src in bytes.chunks(PAGE_SIZE) {
            let vpn = vpn_iter.next().expect("Not enough VPNs for bytes");
            let dst = &mut page_table
                .translate(vpn)
                .expect("failed to translate VPN to PTE")
                .ppn()
                .get_bytes_array_mut()[..src.len()];
            dst.copy_from_slice(src);
        }
    }
}

#[derive(Copy, Clone)]
pub struct VPNRange {
    start: VirtPageNum,
    end: VirtPageNum,
}

impl VPNRange {
    pub fn new(start: VirtPageNum, end: VirtPageNum) -> Self {
        Self { start, end }
    }
}

impl IntoIterator for VPNRange {
    type Item = VirtPageNum;

    type IntoIter = VPNRangeIterator;

    fn into_iter(self) -> Self::IntoIter {
        VPNRangeIterator {
            current: self.start,
            end: self.end,
        }
    }
}

pub struct VPNRangeIterator {
    current: VirtPageNum,
    end: VirtPageNum,
}

impl Iterator for VPNRangeIterator {
    type Item = VirtPageNum;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            None
        } else {
            let cur = self.current;
            self.current.0 += 1; // move VPN
            Some(cur)
        }
    }
}

/// The type of mapping for a memory area.
///
/// - `Identical`: The virtual page number is mapped to the same physical page number.
/// - `Framed`: Each virtual page is mapped to a newly allocated physical frame.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical,
    Framed,
}

bitflags! {
    /// this is subset of PTEFlags for safty concert not export all
    /// control here.
    #[derive(Copy, Clone, PartialEq, Debug)]
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

pub fn remap_kernel_test() {
    let kernel_space = KERNEL_SPACE.exclusive_access();
    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) / 2).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
    assert!(
        !kernel_space
            .page_table
            .translate(mid_text.floor())
            .unwrap()
            .writable()
    );
    assert!(
        !kernel_space
            .page_table
            .translate(mid_rodata.floor())
            .unwrap()
            .writable(),
    );
    assert!(
        !kernel_space
            .page_table
            .translate(mid_data.floor())
            .unwrap()
            .executable(),
    );
    println!("remap_test passed!");
}
