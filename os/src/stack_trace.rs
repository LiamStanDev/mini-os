use core::arch::asm;

// Print kernel stack is unsafe
pub unsafe fn print_stack_trace() {
    let mut fp: *const usize;

    // out(reg) will let compiler choose register to do that
    unsafe {
        asm!("mv {}, fp", out(reg) fp);
    }
    println!("== Begin stack trace ==");
    while !fp.is_null() {
        // NOTE: function call prologue
        // see: https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/5support-func-call.html#term-calling-convention
        // addi sp, sp, -frame_size
        // sd   ra, frame_size-8(sp) // lowest 1
        // sd   fp, frame_size-16(sp) // lowest 2
        // mv   fp, sp
        let saved_ra;
        let saved_fp;
        unsafe {
            saved_ra = *fp.sub(1);
            saved_fp = *fp.sub(2);
        }

        println!("0x{:016x}, fp = 0x{:016x}", saved_ra, saved_fp);
        fp = saved_fp as *const usize;
    }
    println!("== End stack trace ==");
}
