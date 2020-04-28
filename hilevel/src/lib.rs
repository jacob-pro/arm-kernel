#![no_std]
#![feature(alloc_error_handler)]
#![feature(core_intrinsics)]
#![feature(map_first_last)]
#![feature(step_trait)]
#![feature(asm)]

#[macro_use]
extern crate alloc;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod bindings;

mod allocator;
mod io;
mod state;
mod process;
mod util;

use core::panic::PanicInfo;
use bindings::main_console;
use bindings::{UART0, UART1, GICC0, GICD0, TIMER0};
use bindings::{GIC_SOURCE_TIMER0, GIC_SOURCE_UART0, GIC_SOURCE_UART1};
use core::slice;
use core::fmt::Write;
use crate::io::PL011;
use crate::process::{ScheduleSource, Context};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;


#[no_mangle]
#[cfg(not(test))]
pub extern fn hilevel_handler_rst(ctx: *mut Context) {
    let ctx = unsafe { &mut *ctx};
    let state = state::init();

    unsafe {
        (*TIMER0).Timer1Load  = 0x00010000;
        //(*TIMER0).Timer1Load  = 0x00100000; // select period = 2^20 ticks ~= 1 sec
        (*TIMER0).Timer1Ctrl  = 0x00000002; // select 32-bit   timer
        (*TIMER0).Timer1Ctrl |= 0x00000040; // select periodic timer
        (*TIMER0).Timer1Ctrl |= 0x00000020; // enable          timer interrupt
        (*TIMER0).Timer1Ctrl |= 0x00000080; // enable          timer

        (*UART0).IMSC       |= 0x00000010; // enable UART    (Rx) interrupt
        (*UART0).CR          = 0x00000301; // enable UART (Tx+Rx)

        (*UART1).IMSC       |= 0x00000010; // enable UART    (Rx) interrupt
        (*UART1).CR          = 0x00000301; // enable UART (Tx+Rx)

        (*GICC0).PMR          = 0x000000F0; // unmask all            interrupts
        (*GICD0).ISENABLER1  |= 0x00001010; // enable timer + UART   interrupts
        (*GICC0).CTLR         = 0x00000001; // enable GIC interface
        (*GICD0).CTLR         = 0x00000001; // enable GIC distributor

        bindings::int_enable_irq();
    }

    state.process_manager.create_process(main_console, state.io_manager.default_files());
    state.process_manager.dispatch(ctx, ScheduleSource::Reset);
}

#[no_mangle]
#[cfg(not(test))]
pub extern fn hilevel_handler_irq(ctx: *mut Context) {
    let ctx = unsafe { &mut *ctx};
    let state = state::get();

    unsafe {
        // Read  the interrupt identifier so we know the source.
        let id: u32 = (*GICC0).IAR;

        match id {
            GIC_SOURCE_TIMER0 => {
                (*TIMER0).Timer1IntClr = 0x01;
                state.process_manager.dispatch(ctx, ScheduleSource::Timer);
            },
            GIC_SOURCE_UART0 => {
                state.io_manager.uart0_ro.borrow_mut().on_state_change();
                state.io_manager.uart0_wo.borrow_mut().on_state_change();
                state.process_manager.dispatch(ctx, ScheduleSource::Io);
            },
            GIC_SOURCE_UART1 => {
                state.io_manager.uart1_rw.borrow_mut().on_state_change();
                state.process_manager.dispatch(ctx, ScheduleSource::Io);
            }
            _ => {}
        }

        // Write the interrupt identifier to signal we're done.
        (*GICC0).EOIR = id;
    }
}

#[derive(FromPrimitive, PartialEq)]
pub enum SysCall {
    Yield = 0,
    Write = 1,
    Read = 2,
    Fork = 3,
    Exit = 4,
    Exec = 5,
    Kill = 6,
    Nice = 7,
    Close = 8,
    Pipe = 9,
}

const MINUS_ONE: i32 = -1;

#[no_mangle]
#[cfg(not(test))]
pub extern fn hilevel_handler_svc(ctx: *mut Context, id: u32) {
    let ctx = unsafe { &mut *ctx};
    let state = state::get();

    FromPrimitive::from_u32(id).map(|id| {
        match id {
            SysCall::Yield => {/*The scheduler will deal with this further down*/}
            SysCall::Write => {
                let fid = ctx.gpr[0] as i32;
                let start_ptr = ctx.gpr[1] as *const u8;
                let length = ctx.gpr[2] as usize;
                let slice = unsafe { slice::from_raw_parts(start_ptr, length) };
                let current = state.process_manager.current_process().unwrap();
                ctx.gpr[0] = current.borrow_mut().write(fid, slice).map_or(MINUS_ONE as u32, |read| read as u32);
            }
            SysCall::Read => {
                let fid = ctx.gpr[0] as i32;
                let start_ptr = ctx.gpr[1] as *mut u8;
                let length = ctx.gpr[2] as usize;
                let slice = unsafe { slice::from_raw_parts_mut(start_ptr, length) };
                let current = state.process_manager.current_process().unwrap();
                ctx.gpr[0] = current.borrow_mut().read(fid, slice).map_or(MINUS_ONE as u32, |written| written as u32);
            }
            SysCall::Fork => {
                ctx.gpr[0] = state.process_manager.fork(ctx) as u32;
            }
            SysCall::Exit => {
                let code = ctx.gpr[0];
                state.process_manager.exit(code);
            }
            SysCall::Exec => {
                let address = ctx.gpr[0];
                state.process_manager.exec(ctx, address);
            }
            SysCall::Kill => {
                let pid = ctx.gpr[0] as i32;
                let signal = ctx.gpr[1] as i32;
                ctx.gpr[0] = state.process_manager.signal(pid, signal).map_or(MINUS_ONE as u32, |_| 0);
            }
            SysCall::Nice => {/* Unimplemented */}
            SysCall::Close => {
                let fid = ctx.gpr[0] as i32;
                let current = state.process_manager.current_process().unwrap();
                ctx.gpr[0] = current.borrow_mut().close(fid).map_or(MINUS_ONE as u32, |_| 0);
            }
            SysCall::Pipe => {
                let array_ptr = ctx.gpr[0] as *mut u8;
                let _slice = unsafe { slice::from_raw_parts_mut(array_ptr, 2) };
            }
        }
        state.process_manager.dispatch(ctx, ScheduleSource::Svc {id});
    });
}

#[panic_handler]
#[cfg(not(test))]
fn handle_panic(info: &PanicInfo) -> ! {
    writeln!(PL011::UART0(), "\n{}", info).ok();
    abort()
}

#[no_mangle]
#[cfg(not(test))]
pub extern fn abort() -> ! {
    unsafe {
        bindings::int_unable_irq();  // Disable to stop interrupts resuming execution
        core::intrinsics::abort()
    }
}
