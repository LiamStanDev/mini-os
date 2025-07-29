#![no_std]
#![no_main]

use user_lib::{exec, fork, wait, yield_};

#[macro_use]
extern crate user_lib;

#[unsafe(no_mangle)]
fn main() -> i32 {
    // child process
    if fork() == 0 {
        // only pass pointer to os
        exec("user_shell\0");
    } else {
        loop {
            let mut exit_code: i32 = 0;
            let pid = wait(&mut exit_code);
            // child process doesn't exist
            if pid == -1 {
                yield_();
                continue;
            }
            println!(
                "[initproc] Released a zombie process, pid={}, exit_code={}",
                pid, exit_code,
            );
        }
    }
    0
}
