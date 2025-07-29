#![no_std]
#![no_main]

use alloc::string::String;
use user_lib::console::getchar;
use user_lib::{exec, fork, waitpid};

extern crate alloc;

#[macro_use]
extern crate user_lib;

/// Line Feed (LF) ASCII control character (0x0A).
const LF: u8 = 0x0au8;
/// Carriage Return (CR) ASCII control character (0x0D).
const CR: u8 = 0x0du8;
/// Delete (DEL) ASCII control character (0x7F).
const DL: u8 = 0x7fu8;
/// Backspace (BS) ASCII control character (0x08).
const BS: u8 = 0x08u8;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("Rust user shell");
    let mut line: String = String::new();
    print!(">> ");
    loop {
        let c = getchar();
        match c {
            CR | LF => {
                println!("");
                if line.is_empty() {
                    print!(">> ");
                    continue;
                }

                line.push('\0');
                let pid = fork();
                // child process
                if pid == 0 {
                    if exec(line.as_str()) == -1 {
                        println!("Error when executing!");
                        return -4;
                    }
                    unreachable!();
                } else {
                    let mut exit_code: i32 = 0;
                    let exit_pid = waitpid(pid as usize, &mut exit_code);
                    assert_eq!(pid, exit_pid);
                    println!("Shell: Process {} exited with code {}", pid, exit_code);
                }
                line.clear();
            }
            BS | DL => {
                if !line.is_empty() {
                    // move cursor back
                    print!("{}", BS as char);
                    // print the space to overwrite the last character
                    print!(" ");
                    // move cursor back again
                    print!("{}", BS as char);
                    line.pop();
                }
            }
            _ => {
                print!("{}", c as char);
                line.push(c as char);
            }
        }
    }
}
