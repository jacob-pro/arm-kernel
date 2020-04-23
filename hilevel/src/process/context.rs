
const CPSR_USR: u32 = 0x50;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Context {
    pub cpsr: u32,
    pub pc: u32,
    pub gpr: [u32; 13usize],
    pub sp: u32,
    pub lr: u32,
}

impl Context {

    pub fn new(pc: u32, sp: u32) -> Context {
        Context {
            cpsr: CPSR_USR,
            pc,
            gpr: [0; 13],
            sp,
            lr: 0
        }
    }

}
