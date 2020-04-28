use crate::process::{ProcessControlBlock, Context, uninit_bytes, DEFAULT_STACK_BYTES};
use alloc::vec::Vec;
use crate::io::PL011::UART0;
use core::fmt::Write;

extern fn idle_fn() -> ! {
    loop {
        unsafe { asm!("nop"); }
        //write!(UART0(), "nop").ok();
    }
}

pub fn idle_process() -> ProcessControlBlock {
    let stack = uninit_bytes(DEFAULT_STACK_BYTES);
    let tos = stack.last().unwrap() as *const _;
    ProcessControlBlock::new(-1, stack, Context::new(idle_fn as u32, tos as u32) )
}
