#![no_std] // don't link the Rust standard library

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod bindings;

use core::panic::PanicInfo;
use bindings::PL011_putc;
use bindings::UART0;
use bindings::TIMER0;
use bindings::GICC0;
use bindings::GICD0;
use arr_macro::arr;
use core::char::from_digit;
use crate::ProcessStatus::{Executing, Ready};
use core::slice::from_raw_parts;

type PID = cty::c_int;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Context {
    pub cpsr: u32,
    pub pc: u32,
    pub gpr: [u32; 13usize],
    pub sp: u32,
    pub lr: u32,
}

enum ProcessStatus {
    Ready,
    Executing,
}

struct ProcessControlBlock {
    pid: PID,
    status: ProcessStatus,
    top_of_stack: u32,
    context: Context,
}

const CPSR_USR: u32 = 0x50;

static mut EXECUTING: Option<PID> = None;
static mut PROCESS_TABLE: [Option<ProcessControlBlock>; 64] = arr![None; 64];

#[allow(non_upper_case_globals)]
extern {
    fn main_P1();
    fn main_P2();
    fn main_P3();
    fn main_P4();
    static tos_P1: u32;
    static tos_P2: u32;
}


fn dispatch(ctx: &mut Context, prev: &mut Option<ProcessControlBlock>, next: &mut Option<ProcessControlBlock>) {
    let mut prev_pid = '?' as u8;
    let mut next_pid = '?' as u8;

    match prev {
        Some(x) => {
            prev_pid = from_digit(x.pid as u32, 10).unwrap() as u8;
            x.context = *ctx;
        },
        None => {}
    }

    match next {
        Some(x) => {
            next_pid = from_digit(x.pid as u32, 10).unwrap() as u8;
            *ctx = x.context;
        },
        None => {}
    }

    unsafe {
        PL011_putc( UART0, '[' as u8, true );
        PL011_putc( UART0, prev_pid, true );
        PL011_putc( UART0, '-' as u8, true );
        PL011_putc( UART0, '>' as u8, true );
        PL011_putc( UART0, next_pid, true );
        PL011_putc( UART0, ']' as u8, true );
    }

    unsafe { EXECUTING = next.as_ref().map(|x| x.pid) }; // update   executing process to P_{next}
}

fn schedule(ctx: &mut Context) {
    unsafe {
        match EXECUTING {
            Some(x) => {

                if x == PROCESS_TABLE[0].as_ref().unwrap().pid {
                    dispatch( ctx, &mut PROCESS_TABLE[ 0 ], &mut PROCESS_TABLE[ 1 ] );  // context switch P_1 -> P_2
                    PROCESS_TABLE[0].as_mut().unwrap().status = Ready;             // update   execution status  of P_1
                    PROCESS_TABLE[1].as_mut().unwrap().status = Executing;         // update   execution status  of P_2

                } else if x == PROCESS_TABLE[1].as_ref().unwrap().pid {
                    dispatch( ctx, &mut PROCESS_TABLE[ 1 ], &mut PROCESS_TABLE[ 0 ] );  // context switch P_2 -> P_1
                    PROCESS_TABLE[1].as_mut().unwrap().status = Ready;             // update   execution status  of P_2
                    PROCESS_TABLE[0].as_mut().unwrap().status = Executing;         // update   execution status  of P_1
                }
            }
            None => {}
        }
    }
}


#[no_mangle]
pub extern fn hilevel_handler_rst(ctx: *mut Context) {
    let ctx = unsafe { &mut *ctx};

    unsafe {
        let tos = &tos_P1 as *const _ as u32;
        PROCESS_TABLE[0] = Some(ProcessControlBlock {
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
        PROCESS_TABLE[1] = Some(ProcessControlBlock {
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

    unsafe {
        dispatch(ctx, &mut None, &mut PROCESS_TABLE[0]);
    }
}

#[no_mangle]
pub extern fn hilevel_handler_irq(ctx: *mut Context) {
    let ctx = unsafe { &mut *ctx};

    unsafe {
        let id: u32 = (*GICC0).IAR;

        if id == bindings::GIC_SOURCE_TIMER0 {

            PL011_putc(UART0, 'T' as u8, true);
            (*TIMER0).Timer1IntClr = 0x01;
            schedule(ctx);
        }

        (*GICC0).EOIR = id;
    }

}

#[no_mangle]
pub extern fn hilevel_handler_svc(ctx: *mut Context, id: u32) {
    let ctx = unsafe { &mut *ctx};

    match id {
        0 => {
            schedule(ctx);
        },
        1 => {
            let _file_descriptor = ctx.gpr[0];
            let start_ptr = ctx.gpr[1] as *const u8;
            let length = ctx.gpr[2] as usize;
            let slice = unsafe { from_raw_parts(start_ptr, length) };
            slice.iter().for_each(|b| {
                unsafe { PL011_putc( UART0, *b, true ) };
            });
        }
        _ => {}
    }
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
