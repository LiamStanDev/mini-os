pub const MAX_APP_NUM: usize = 4;
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000; // 128 KiB

pub const USER_STACK_SIZE: usize = 4096; // 4 KiB
pub const KERNEL_STACK_SIZE: usize = 4096 * 2; // 8 KiB
pub const CLOCK_FREQ: u64 = 12500000;
