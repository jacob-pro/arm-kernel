#![no_std]
#![feature(alloc_error_handler)]
#![feature(core_intrinsics)]

extern crate alloc;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod bindings;
mod allocator;
mod device;
mod state;
mod process;

use core::panic::PanicInfo;
use bindings::PL011_putc;
use bindings::TIMER0;
use bindings::GICC0;
use bindings::GICD0;
use core::slice::from_raw_parts;
use core::fmt::Write;
use crate::device::PL011::UART0;
use process::*;


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Context {
    pub cpsr: u32,
    pub pc: u32,
    pub gpr: [u32; 13usize],
    pub sp: u32,
    pub lr: u32,
}

const CPSR_USR: u32 = 0x50;


#[allow(non_upper_case_globals)]
extern {
    fn main_P1();
    fn main_P2();
    fn main_P3();
    fn main_P4();
    static tos_P1: u32;
    static tos_P2: u32;
}


#[no_mangle]
pub extern fn hilevel_handler_rst(ctx: *mut Context) {
    let ctx = unsafe { &mut *ctx};
    let state = state::init();

    unsafe {
        let tos = &tos_P1 as *const _ as u32;
        state.process_manager.table.insert(1, ProcessControlBlock {
            pid: 1,
            status: ProcessStatus::Ready,
            top_of_stack: tos,
            context: Context {
                cpsr: CPSR_USR,
                pc: main_P3 as u32,
                gpr: [0; 13],
                sp: tos,
                lr: 0,
            }
        });
    }
    unsafe {
        let tos = &tos_P2 as *const _ as u32;
        state.process_manager.table.insert(2, ProcessControlBlock {
            pid: 2,
            status: ProcessStatus::Ready,
            top_of_stack: tos,
            context: Context {
                cpsr: CPSR_USR,
                pc: main_P4 as u32,
                gpr: [0; 13],
                sp: tos,
                lr: 0,
            }
        });
    }

    unsafe {
        (*TIMER0).Timer1Load  = 0x00100000; // select period = 2^20 ticks ~= 1 sec
        (*TIMER0).Timer1Ctrl  = 0x00000002; // select 32-bit   timer
        (*TIMER0).Timer1Ctrl |= 0x00000040; // select periodic timer
        (*TIMER0).Timer1Ctrl |= 0x00000020; // enable          timer interrupt
        (*TIMER0).Timer1Ctrl |= 0x00000080; // enable          timer

        (*GICC0).PMR          = 0x000000F0; // unmask all            interrupts
        (*GICD0).ISENABLER1  |= 0x00000010; // enable timer          interrupt
        (*GICC0).CTLR         = 0x00000001; // enable GIC interface
        (*GICD0).CTLR         = 0x00000001; // enable GIC distributor

        bindings::int_enable_irq();
    }


    state.process_manager.dispatch(ctx, None, Some(1));

}

#[no_mangle]
pub extern fn hilevel_handler_irq(ctx: *mut Context) {
    let ctx = unsafe { &mut *ctx};
    let state = state::get();

    unsafe {
        let id: u32 = (*GICC0).IAR;

        if id == bindings::GIC_SOURCE_TIMER0 {

            PL011_putc(bindings::UART0, 'T' as u8, true);
            (*TIMER0).Timer1IntClr = 0x01;
            state.process_manager.schedule(ctx);
        }

        (*GICC0).EOIR = id;
    }

}

#[no_mangle]
pub extern fn hilevel_handler_svc(ctx: *mut Context, id: u32) {
    let ctx = unsafe { &mut *ctx};
    let state = state::get();

    match id {
        0 => {
            state.process_manager.schedule(ctx);
        },
        1 => {
            let _file_descriptor = ctx.gpr[0];
            let start_ptr = ctx.gpr[1] as *const u8;
            let length = ctx.gpr[2] as usize;
            let slice = unsafe { from_raw_parts(start_ptr, length) };
            slice.iter().for_each(|b| {
                unsafe { PL011_putc( bindings::UART0, *b, true ) };
            });
        }
        _ => {}
    }
}


#[panic_handler]
fn handle_panic(info: &PanicInfo) -> ! {
    writeln!(UART0(), "\n{}", info).ok();
    abort()
}

#[no_mangle]
pub extern fn abort() -> ! {
    unsafe {
        // Disable to stop interrupts resuming execution
        bindings::int_unable_irq();
        core::intrinsics::abort()
    }
}
