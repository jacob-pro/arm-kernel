mod table;
mod scheduler;

use crate::Context;
use crate::device::PL011::UART0;
use core::fmt::Write;
use alloc::string::ToString;
use alloc::vec::Vec;
use crate::process::table::{ProcessTable, ProcessTableMethods};
use alloc::rc::{Rc, Weak};
use core::cell::{RefCell, RefMut};
use crate::process::scheduler::MLFQ;

pub type PID = u32;

const DEFAULT_STACK_BYTES: usize = 0x00001000; // = 4 KiB

#[derive(Default)]
pub struct ProcessManager {
    table: ProcessTable,
    scheduler: MLFQ,
}

pub enum ProcessStatus {
    Ready,
    Executing,
    Terminated
}

pub enum ScheduleSource {
    Svc {id: u32},
    Timer,
    Reset,
}

pub type StrongPcbRef = Rc<RefCell<ProcessControlBlock>>;
pub type WeakPcbRef = Weak<RefCell<ProcessControlBlock>>;

pub struct ProcessControlBlock {
    pub pid: PID,
    pub status: ProcessStatus,
    pub stack: Vec<u8>,
    context: Context,
}

const CPSR_USR: u32 = 0x50;

impl ProcessControlBlock {

    fn new(pid: PID, stack: Vec<u8>, main: unsafe extern fn()) -> ProcessControlBlock {
        // last() because the stack grows downwards from higher -> lower addresses
        let tos = stack.last().expect("Stack needs to be larger than 0") as *const _;
        ProcessControlBlock{
            pid,
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

    pub fn create_process(&mut self, main: unsafe extern fn()) -> PID {
        let pid = self.table.new_pid();
        let stack = uninit_bytes(DEFAULT_STACK_BYTES);
        let process = Rc::new(RefCell::new(ProcessControlBlock::new(pid, stack, main)));
        self.table.insert(pid, Rc::clone(&process));
        self.scheduler.insert_process(Rc::downgrade(&process));
        pid
    }

    #[inline(always)]
    pub fn schedule(&mut self, ctx: &mut Context, src: ScheduleSource) {
        self.scheduler.schedule(src, |a, b| {
            dispatch(ctx, a, b);
        });
    }

}

// A heap allocated byte array of length size. Values are uninitialised
fn uninit_bytes(size: usize) -> Vec<u8> {
    let mut stack: Vec<u8> = Vec::with_capacity(size);
    unsafe { stack.set_len(size) };
    stack
}

fn dispatch(ctx: &mut Context, prev: Option<RefMut<ProcessControlBlock>>, mut next: RefMut<ProcessControlBlock>) {

    let prev_pid_str = match prev {
        Some(mut x) => {
            x.context = *ctx;
            x.status = ProcessStatus::Ready;
            x.pid.to_string()
        },
        None => {
            "?".to_string()
        }
    };

    *ctx = next.context;
    next.status = ProcessStatus::Executing;

    write!(UART0(), "[{}->{}]", prev_pid_str, next.pid).ok();
}
