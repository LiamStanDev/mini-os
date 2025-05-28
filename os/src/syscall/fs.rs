const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let len = match fd {
        FD_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).expect("invalid UTF-8 encoding");
            print!("{}", str);
            len as isize
        }

        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    };

    len
}
