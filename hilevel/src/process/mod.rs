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

    fn new1(pid: PID, stack: Vec<u8>, context: Context) -> ProcessControlBlock {
        let tos = stack.last().unwrap() as *const _;
        let bos = stack.first().unwrap() as *const _;
        assert!(context.sp <= tos as u32);
        assert!(context.sp >= bos as u32);
        ProcessControlBlock{
            pid,
            status: ProcessStatus::Ready,
            stack,
            context,
        }
    }

    fn new(pid: PID, stack: Vec<u8>, pc: u32, sp: u32) -> ProcessControlBlock {
        let tos = stack.last().unwrap() as *const _;
        let bos = stack.first().unwrap() as *const _;
        assert!(sp <= tos as u32);
        assert!(sp >= bos as u32);
        ProcessControlBlock{
            pid,
            status: ProcessStatus::Ready,
            stack,
            context: Context {
                cpsr: CPSR_USR,
                pc,
                gpr: [0; 13],
                sp,
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
        let tos = stack.last().unwrap() as *const _;         // last() because the stack grows downwards from higher -> lower addresses
        let pcb = ProcessControlBlock::new(pid, stack, main as u32, tos as u32);
        let process = Rc::new(RefCell::new(pcb));
        self.table.insert(pid, Rc::clone(&process));
        self.scheduler.insert_process(Rc::downgrade(&process));
        pid
    }

    pub fn _signal_process(&mut self, pid: PID) -> Result<(), String> {
        let x = self.table.remove(&pid).ok_or("PID not found")?;
        x.borrow_mut().status = ProcessStatus::Terminated;
        Ok(())
    }

    // Forks current process, returns the child PID
    pub fn fork(&mut self, ctx: &Context) -> PID {
        let current = self.scheduler.current_process();
        let borrowed = current.borrow();
        let new_pid = self.table.new_pid();
        let new_stack = borrowed.stack.clone();
        let new_sp = adjust_sp(&borrowed.stack, &new_stack, ctx.sp);
        let mut new_ctx = ctx.clone();
        new_ctx.sp = new_sp;
        new_ctx.gpr[0] = 0;
        let pcb = ProcessControlBlock::new1(new_pid, new_stack, new_ctx);
        let process = Rc::new(RefCell::new(pcb));
        self.table.insert(new_pid, Rc::clone(&process));
        self.scheduler.insert_process(Rc::downgrade(&process));
        return new_pid
    }

    // Change current process to new PC address
    pub fn exec(&mut self, ctx: &mut Context, address: u32) {
        let current = self.scheduler.current_process();
        let borrowed = current.borrow_mut();
        let tos = borrowed.stack.last().unwrap() as *const _;
        *ctx = Context {
            cpsr: CPSR_USR,
            pc: address,
            gpr: [0; 13],
            sp: tos as u32,
            lr: 0
        };
    }

    // Exits current process
    pub fn exit(&mut self, _code: u32) {
        let current = self.scheduler.current_process();
        let mut borrowed = current.borrow_mut();
        borrowed.status = ProcessStatus::Terminated;
        self.table.remove(&borrowed.pid);
        write!(UART0(), "[{} Exited]", borrowed.pid).ok();
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


fn adjust_sp(old_stack: &Vec<u8>, new_stack: &Vec<u8>, old_sp: u32) -> u32 {
    let old_tos = old_stack.last().unwrap() as *const _;
    let diff = old_tos as u32 - old_sp;
    let new_tos = new_stack.last().unwrap() as *const _;
    new_tos as u32 - diff
}
