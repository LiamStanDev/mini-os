use core::arch::asm;

use lazy_static::*;
use log::{info, trace};

use crate::sbi::shutdown;
use crate::sync::UPSafeCell;
use crate::trap::context::TrapContext;

const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000; // 128 KiB

const USER_STACK_SIZE: usize = 4096 * 2; // 8 KiB
const KERNEL_STACK_SIZE: usize = 4096 * 2; // 8 KiB

static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};

static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
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

lazy_static! {
    static ref APP_MANAGER: UPSafeCell<AppManager> = {
        unsafe extern "C" {
            safe fn _num_app();
        }
        let num_app_ptr = _num_app as usize as *const usize;

        let num_app;
        let mut app_start_addr: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];

        unsafe {
            num_app = num_app_ptr.read_volatile();
            let app_start_addr_raw: &[usize] =
                core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
            app_start_addr[..=num_app].copy_from_slice(app_start_addr_raw);

            UPSafeCell::new(AppManager {
                num_app,
                current_app: 0,
                app_start_addr,
            })
        }
    };
}

struct AppManager {
    num_app: usize,
    current_app: usize,
    app_start_addr: [usize; MAX_APP_NUM + 1], // last addr is the end of app addr
}

impl AppManager {
    pub fn print_app_info(&self) {
        info!("[kernel] num_app = {}", self.num_app);
        for i in 0..self.num_app {
            info!(
                "[kernel] app_{} [{:#x}, {:#x})",
                i,
                self.app_start_addr[i],
                self.app_start_addr[i + 1]
            );
        }
    }

    pub fn get_current_app(&self) -> usize {
        self.current_app
    }

    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }

    fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            println!("All applications completed!");
            shutdown(false);
        }

        trace!("[kernel] Loading app_{}", app_id);

        unsafe {
            // clear app area
            core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT).fill(0);

            // copy new app into app area
            let app_src = core::slice::from_raw_parts(
                self.app_start_addr[app_id] as *const u8,
                self.app_start_addr[app_id + 1] - self.app_start_addr[app_id],
            );

            let app_dst =
                core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
            app_dst.copy_from_slice(app_src);

            trace!(
                "[kernel] app_{} copied {} bytes from {:#x} to {:#x}",
                app_id,
                app_src.len(),
                self.app_start_addr[app_id],
                APP_BASE_ADDRESS
            );

            // after update .text section, we need to clear i-cache (instruction cache) to
            // fetch new app instruction.
            asm!("fence.i");
        }
    }
}

pub(crate) fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}

pub(crate) fn init() {
    // first use APP_MANAGER, so it initialize here
    print_app_info();
}

pub(crate) fn run_next_app() -> ! {
    let mut app_manager_ref = APP_MANAGER.exclusive_access();
    let current_app = app_manager_ref.current_app;

    app_manager_ref.load_app(current_app);
    app_manager_ref.move_to_next_app();
    drop(app_manager_ref); // because exclusive access

    unsafe extern "C" {
        fn __restore(cx_addr: usize);
    }

    let new_app_ctx = TrapContext::init_app_ctx(APP_BASE_ADDRESS, USER_STACK.get_sp());
    let cx_addr = KERNEL_STACK.push_trap_ctx(new_app_ctx) as *const _ as usize;

    trace!(
        "[kernel] Jumping to user app at {:x}, trap context at {:x}",
        APP_BASE_ADDRESS, cx_addr
    );

    unsafe {
        __restore(cx_addr);
    }

    panic!("Unreachable in batch::run_current_app!");
}
