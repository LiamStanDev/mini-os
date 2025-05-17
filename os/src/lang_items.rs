use core::panic::PanicInfo;

use crate::sbi::shutdown;
use crate::stack_trace::print_stack_trace;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        println!("Panicked: {}", info.message());
    }
    unsafe {
        print_stack_trace();
    }
    shutdown(true)
}
