use crate::Context;
use alloc::collections::BTreeMap;
use core::char::from_digit;
use crate::bindings::PL011_putc;
use crate::bindings;

pub type PID = u32;
const CPSR_USR: u32 = 0x50;

#[derive(Default)]
pub struct ProcessManager {
    pub table: BTreeMap<PID, ProcessControlBlock>,
    pub executing: Option<PID>,
}

pub enum ProcessStatus {
    Ready,
    Executing,
}

pub struct ProcessControlBlock {
    pub pid: PID,
    pub status: ProcessStatus,
    pub top_of_stack: u32,
    pub context: Context,
}


impl ProcessManager {

    pub fn schedule(&mut self, ctx: &mut Context) {
        match self.executing {
            Some(x) => {

                if x == self.table[&1].pid {
                    self.dispatch( ctx, Some(1), Some(2) );  // context switch P_1 -> P_2
                    self.table.get_mut(&1).unwrap().status = ProcessStatus::Ready;             // update   execution status  of P_1
                    self.table.get_mut(&2).unwrap().status = ProcessStatus::Executing;         // update   execution status  of P_2

                } else if x == self.table[&2].pid {
                    self.dispatch( ctx, Some(2), Some(1) );  // context switch P_2 -> P_1
                    self.table.get_mut(&2).unwrap().status = ProcessStatus::Ready;             // update   execution status  of P_2
                    self.table.get_mut(&1).unwrap().status = ProcessStatus::Executing;         // update   execution status  of P_1
                }
            }
            None => {
                self.dispatch( ctx, None, Some(1) );  // context switch P_1 -> P_2
                self.table.get_mut(&1).unwrap().status = ProcessStatus::Executing;             // update   execution status  of P_1
                self.table.get_mut(&2).unwrap().status = ProcessStatus::Ready;         // update   execution status  of P_2
            }
        }
    }

    fn dispatch(&mut self, ctx: &mut Context, prev: Option<PID>, next: Option<PID>) {
        let mut prev_pid = '?' as u8;
        let mut next_pid = '?' as u8;

        match prev {
            Some(x) => {
                prev_pid = from_digit(x, 10).unwrap() as u8;
                self.table.get_mut(&x).unwrap().context = *ctx;
            },
            None => {}
        }

        match next {
            Some(x) => {
                next_pid = from_digit(x, 10).unwrap() as u8;
                *ctx = self.table.get_mut(&x).unwrap().context;
            },
            None => {}
        }

        unsafe {
            PL011_putc( bindings::UART0, '[' as u8, true );
            PL011_putc( bindings::UART0, prev_pid, true );
            PL011_putc( bindings::UART0, '-' as u8, true );
            PL011_putc( bindings::UART0, '>' as u8, true );
            PL011_putc( bindings::UART0, next_pid, true );
            PL011_putc( bindings::UART0, ']' as u8, true );
        }

        self.executing = next; // update   executing process to P_{next}
    }

    pub fn create_process(&mut self, pid: PID, tos: *const cty::c_void, main: unsafe extern fn()) {

        self.table.insert(pid, ProcessControlBlock {
            pid: pid,
            status: ProcessStatus::Ready,
            top_of_stack: tos as u32,
            context: Context {
                cpsr: CPSR_USR,
                pc: main as u32,
                gpr: [0; 13],
                sp: tos as u32,
                lr: 0,
            }
        });


    }

}

