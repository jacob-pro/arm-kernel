mod table;
mod scheduler;

use crate::{Context, SysCall};
use crate::device::PL011::UART0;
use core::fmt::Write;
use alloc::string::{ToString, String};
use alloc::vec::Vec;
use crate::process::table::ProcessTable;
use alloc::rc::{Rc, Weak};
use core::cell::RefCell;
use crate::process::scheduler::MLFQScheduler;

pub type PID = u32;

const DEFAULT_STACK_BYTES: usize = 0x00001000; // = 4 KiB

#[derive(Default)]
pub struct ProcessManager {
    table: ProcessTable,
    scheduler: MLFQScheduler,
}

#[derive(PartialEq)]
pub enum ProcessStatus {
    Ready,
    Executing,
    Terminated
}

pub enum ScheduleSource {
    Svc {id: SysCall},
    Timer,
    Reset,
}

pub type StrongPcbRef = Rc<RefCell<ProcessControlBlock>>;
pub type WeakPcbRef = Weak<RefCell<ProcessControlBlock>>;

pub struct ProcessControlBlock {
    pid: PID,
    status: ProcessStatus,
    stack: Vec<u8>,
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

    // Create a new process
    pub fn create_process(&mut self, main: unsafe extern fn()) -> PID {
        let pid = self.table.new_pid();
        let stack = uninit_bytes(DEFAULT_STACK_BYTES);
        let process = Rc::new(RefCell::new(ProcessControlBlock::new(pid, stack, main)));
        self.table.insert(pid, Rc::clone(&process));
        self.scheduler.insert_process(Rc::downgrade(&process));
        pid
    }

    // Kills another process
    // We only need to remove from process table, the scheduler only keeps a weak reference
    pub fn _kill_process(&mut self, pid: PID) -> Result<(), String> {
        let x = self.table.remove(&pid).ok_or("PID not found")?;
        x.borrow_mut().status = ProcessStatus::Terminated;
        Ok(())
    }

    // Exits current process
    pub fn _exit(&mut self, _code: u32) {
        let x = self.scheduler.current_process();
        x.borrow_mut().status = ProcessStatus::Terminated;
        self.table.remove(&x.borrow().pid);
    }

    pub fn dispatch(&mut self, ctx: &mut Context, src: ScheduleSource) {
        self.scheduler.schedule(src, |prev, mut next| {

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
        });
    }

}

// A heap allocated byte array of length size. Values are uninitialised
fn uninit_bytes(size: usize) -> Vec<u8> {
    let mut stack: Vec<u8> = Vec::with_capacity(size);
    unsafe { stack.set_len(size) };
    stack
}
