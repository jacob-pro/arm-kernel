#![no_std] // don't link the Rust standard library

use core::panic::PanicInfo;

extern {
    fn hilevel_handler_rst_c();
}

#[no_mangle]
pub extern "C" fn hilevel_handler_rst() {
    unsafe { hilevel_handler_rst_c() }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
