mod table;

use crate::Context;
use alloc::collections::BTreeMap;
use crate::device::PL011::UART0;
use core::fmt::Write;
use alloc::string::ToString;
use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::slice;
use crate::process::table::{ProcessTable, ProcessTableMethods};

pub type PID = u32;

const DEFAULT_STACK_BYTES: usize = 0x00001000; // = 4 KiB

#[derive(Default)]
pub struct ProcessManager {
    pub table: ProcessTable,
    pub executing: Option<PID>,
}

#[derive(Debug, Clone)]
pub enum ProcessStatus {
    Ready,
    Executing,
}

pub enum ScheduleSource {
    Svc {id: u32},
    Timer,
    Reset,
}

#[derive(Debug, Clone)]
pub struct ProcessControlBlock {
    pub status: ProcessStatus,
    pub stack: Vec<u8>,
    pub context: Context,
}

const CPSR_USR: u32 = 0x50;

impl ProcessControlBlock {

    fn new(stack: Vec<u8>, main: unsafe extern fn()) -> ProcessControlBlock {
        let tos = stack.last().expect("Stack needs to be longer than 0") as *const _;
        ProcessControlBlock{
            status: ProcessStatus::Ready,
            stack,
            context: Context {
                cpsr: CPSR_USR,
                pc: main as u32,
                gpr: [0; 13],
                sp: tos as u32,
                lr: 0
            }
        }
    }
}


impl ProcessManager {

    pub fn schedule(&mut self, ctx: &mut Context, src: ScheduleSource) {
        match self.executing {
            Some(x) => {

                if x == 0 {
                    self.dispatch( ctx, Some(0), 1 );  // context switch P_1 -> P_2

                } else if x == 1 {
                    self.dispatch( ctx, Some(1), 0 );  // context switch P_2 -> P_1
                }
            }
            None => {
                self.dispatch( ctx, None, 0 );  // context switch P_1 -> P_2
            }
        }
    }

    fn dispatch(&mut self, ctx: &mut Context, prev_pid: Option<PID>, next_pid: PID) {

        let prev_pid_str = match prev_pid {
            Some(x) => {
                let prev = self.table.get_mut(&x).unwrap();
                prev.context = *ctx;
                prev.status = ProcessStatus::Ready;
                x.to_string()
            },
            None => {
                "?".to_string()
            }
        };

        let next = self.table.get_mut(&next_pid).unwrap();
        *ctx = next.context;
        next.status = ProcessStatus::Executing;

        write!(UART0(), "[{}->{}]", prev_pid_str, next_pid).ok();

        self.executing = Some(next_pid); // update   executing process to P_{next}
    }

    pub fn create_process(&mut self, main: unsafe extern fn()) -> PID {

        let pid = self.table.new_pid();
        let stack = uninit_bytes(DEFAULT_STACK_BYTES);
        let process = ProcessControlBlock::new(stack, main);
        self.table.insert(pid, process);
        pid
    }

}

// A heap allocated byte array of length size. Values are uninitialised
fn uninit_bytes(size: usize) -> Vec<u8> {
    let mut stack: Vec<u8> = Vec::with_capacity(size);
    unsafe { stack.set_len(size) };
    stack
}

