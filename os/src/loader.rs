use crate::config::*;
use crate::trap::context::TrapContext;
use core::arch::asm;

static KERNEL_STACKS: [KernelStack; MAX_APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_APP_NUM];

static USER_STACKS: [UserStack; MAX_APP_NUM] = [UserStack {
    data: [0; USER_STACK_SIZE],
}; MAX_APP_NUM];

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

impl KernelStack {
    fn get_sp(&self) -> usize {
        // stack grow downard. the pointer to data is at buttom of the stack
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    // why &'static?
    // because it push to KERNEL_STACK (static)
    fn push_trap_ctx(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
        }
        unsafe { cx_ptr.as_mut().unwrap() }
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        // stack grow downard. the pointer to data is at buttom of the stack
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

pub(crate) fn get_num_app() -> usize {
    unsafe extern "C" {
        safe fn _num_app();
    }

    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

fn get_app_entry(app_id: usize) -> usize {
    APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

pub(crate) fn load_apps() {
    unsafe extern "C" {
        safe fn _num_app();
    }

    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();

    // get app address
    let app_start_addrs = unsafe {
        // include the latest app end addr
        core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1)
    };

    for i in 0..num_app {
        let app_entry_addr = get_app_entry(i);

        // clear app space
        unsafe {
            core::slice::from_raw_parts_mut(app_entry_addr as *mut u8, APP_SIZE_LIMIT).fill(0);
        }

        // load app
        let src = unsafe {
            core::slice::from_raw_parts(
                app_start_addrs[i] as *const u8,
                app_start_addrs[i + 1] - app_start_addrs[i],
            )
        };
        let dst = unsafe { core::slice::from_raw_parts_mut(app_entry_addr as *mut u8, src.len()) };
        dst.copy_from_slice(src);

        unsafe { asm!("fence.i") }
    }
}

pub(crate) fn init_app_ctx(app_id: usize) -> &'static mut TrapContext {
    KERNEL_STACKS[app_id].push_trap_ctx(TrapContext::init_app_ctx(
        get_app_entry(app_id),
        USER_STACKS[app_id].get_sp(),
    ))
}
