#![no_std] // don't link the Rust standard library

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod bindings;

use core::panic::PanicInfo;
use bindings::hilevel_handler_rst_c;
use bindings::hilevel_handler_irq_c;
use bindings::hilevel_handler_svc_c;
use bindings::ctx_t;

#[no_mangle]
pub extern fn hilevel_handler_rst(ctx: *mut ctx_t) {
    unsafe { hilevel_handler_rst_c(ctx) }
}

#[no_mangle]
pub extern fn hilevel_handler_irq() {
    unsafe { hilevel_handler_irq_c() }
}

#[no_mangle]
pub extern fn hilevel_handler_svc(ctx: *mut ctx_t, id: cty::uint32_t) {
    unsafe { hilevel_handler_svc_c(ctx, id) }
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
