use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;
use riscv::register::time;

const TICKS_PER_SEC: u64 = 100;
const MSEC_PER_SEC: u64 = 1000;

pub fn get_time() -> u64 {
    time::read64()
}

#[allow(unused)]
pub fn get_time_ms() -> u64 {
    time::read64() / (CLOCK_FREQ / MSEC_PER_SEC)
}

pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}
