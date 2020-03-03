#![no_std] // don't link the Rust standard library

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::panic::PanicInfo;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[no_mangle]
pub extern "C" fn hilevel_handler_rst() {
    unsafe { hilevel_handler_rst_c() }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
