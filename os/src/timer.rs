use riscv::register::time;

use crate::board::CLOCK_FREQ;
use crate::sbi::set_timer;

const TICKS_PER_SEC: u64 = 500;
const MICRO_PER_SEC: u64 = 1_000_000;

pub fn get_time() -> u64 {
    time::read64()
}

#[allow(unused)]
pub fn get_time_ms() -> u64 {
    time::read64() / (CLOCK_FREQ / MICRO_PER_SEC)
}

pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}
