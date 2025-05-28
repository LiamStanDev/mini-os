/// The clock frequency of the QEMU board in Hz.
pub const CLOCK_FREQ: u64 = 12_500_000;

/// The end address of the physical memory available to the QEMU board.
/// This constant defines the upper boundary of usable RAM.
/// 0x8800_0000 = 0x8000_0000 + 0x0800_0000 (128MB)
pub const MEMORY_END: usize = 0x8800_0000;
