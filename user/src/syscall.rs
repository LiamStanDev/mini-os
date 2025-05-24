use core::arch::asm;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YEILD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;

    // asm macro can manipulate variable in this context so it
    // should be use inside unsafe block. Note: global_asm! don't
    unsafe {
        // see: https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/5support-func-call.html#term-calling-convention
        asm!(
            "ecall",
            inlateout("a0") args[0] => ret, // x10
            in("a1") args[1], // x11
            in("a2") args[2], // x12
            in("a7") id // x17
        );
    }

    ret
}

pub(crate) fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub(crate) fn sys_exit(exit_code: i32) -> isize {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0])
}

pub(crate) fn sys_yield() -> isize {
    syscall(SYSCALL_YEILD, [0; 3])
}

pub fn sys_get_time() -> isize {
    syscall(SYSCALL_GET_TIME, [0, 0, 0])
}
