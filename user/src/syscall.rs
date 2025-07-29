use core::arch::asm;

const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;

/// Performs a system call with the given ID and arguments.
///
/// # Arguments
///
/// * `id` - The system call number.
/// * `args` - An array of up to three arguments for the system call.
fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}

/// Read data from file into buffer.
///
/// # Arguments
///
/// * `fd` - File descriptor to read from.
/// * `buffer` - Buffer containing data to be read.
///
/// # Returns
///
/// The number of bytes written, or -1 when error.
pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(
        SYSCALL_READ,
        [fd, buffer.as_mut_ptr() as usize, buffer.len()],
    )
}

/// Writes the contents of a buffer to the file descriptor `fd`.
///
/// # Arguments
///
/// * `fd` - File descriptor to write to.
/// * `buffer` - Buffer containing data to write.
///
/// # Returns
///
/// The number of bytes written, or -1 when error.
pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

/// Exits the current process with the given exit code.
///
/// # Arguments
///
/// * `exit_code` - The exit code for the process.
///
/// # Returns
///
/// This function does not return.
pub fn sys_exit(exit_code: i32) -> isize {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0])
}

/// Yields the CPU to another process.
///
/// # Returns
///
/// 0 on success, or a negative error code.
pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

/// Gets the current system time.
///
/// # Returns
///
/// The current time, or a negative error code.
pub fn sys_get_time() -> isize {
    syscall(SYSCALL_GET_TIME, [0, 0, 0])
}

/// Creates a new process by duplicating current process.
///
/// # Returns
///
/// The child's PID in the parent, and 0 in the child.
pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0, 0, 0])
}

/// Waits for a child process to become a zombie, reclaims all its resources, and collects its return value.
///
/// # Arguments
///
/// * `pid` - The process ID of the child to wait for. If set to -1, waits for any child process.
/// * `exit_code` - The address to store the child's return value. If this address is 0, the return value is not saved.
///
/// # Returns
///
/// Returns -1 if the specified child does not exist; returns -2 if none of the specified children have exited yet;
/// otherwise, returns the PID of the child that has exited.
pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0])
}

/// Clear current process address space and load a specified program into it.
///
/// # Arguments
///
/// * `path` - The excutable path.
///
/// Returns
///
/// Returns -1 if error, otherwise no return.
pub fn sys_exec(path: &str) -> isize {
    syscall(SYSCALL_EXEC, [path.as_ptr() as usize, 0, 0])
}
