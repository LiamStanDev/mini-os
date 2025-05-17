use riscv::register::sstatus::{self, SPP, Sstatus};

#[repr(C)]
pub(crate) struct TrapContext {
    pub x: [usize; 32],   // x1-x31
    pub sstatus: Sstatus, // store info of current previlege level
    pub sepc: usize,      // store address of ecall (for purpose of sret)
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    // This method is called when run_next_app, then call __restore(sret) back to user space.
    // Initialize empty app_ctx
    pub(crate) fn init_app_ctx(entry: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut ctx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry, // entry point of app
        };
        ctx.set_sp(sp); // app's user stack pointer
        ctx // return initial Trap Context of app
    }
}
