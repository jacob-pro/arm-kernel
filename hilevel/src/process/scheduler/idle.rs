use crate::process::{ProcessControlBlock, Context};
use alloc::vec::Vec;

extern fn idle_fn() -> ! {
    loop { unsafe { asm!("nop"); } }
}

// A process that does nothing, implementation does not require a stack
pub fn idle_process() -> ProcessControlBlock {
    ProcessControlBlock::new(-1, Vec::new(), Context::new(idle_fn as u32, 0 as u32), Default::default() )
}
