mod scheduler;
mod context;

pub use context::Context;

use crate::SysCall;
use crate::io::PL011::{UART0};
use core::fmt::Write;
use alloc::string::{ToString, String};
use alloc::vec::Vec;
use alloc::rc::Rc;
use core::cell::RefCell;
use crate::process::scheduler::MLFQScheduler;
use crate::io::{FileError, StrongFileDescriptorRef};
use crate::util::IdTable;

pub type PID = i32;
pub type FidTable = IdTable<i32, StrongFileDescriptorRef>;

const DEFAULT_STACK_BYTES: usize = 0x00001000; // = 4 KiB

#[derive(Default)]
pub struct ProcessManager {
    table: IdTable<PID, StrongPcbRef>,
    scheduler: MLFQScheduler,
}

#[derive(Clone, PartialEq)]
pub enum ProcessStatus {
    Ready,
    Executing,
    Exited,
    Terminated,
    Blocked,
}

pub enum ScheduleSource {
    Svc {id: SysCall},
    Timer,
    Reset,
    Io,
}

pub type StrongPcbRef = Rc<RefCell<ProcessControlBlock>>;

pub struct ProcessControlBlock {
    pid: PID,
    status: ProcessStatus,
    stack: Vec<u8>,
    context: Context,
    file_descriptors: FidTable,
}

impl ProcessControlBlock {

    fn new(pid: PID, stack: Vec<u8>, context: Context, file_descriptors: FidTable) -> ProcessControlBlock {
        let tos = stack.last().unwrap() as *const _;
        let bos = stack.first().unwrap() as *const _;
        assert!(context.sp <= tos as u32);
        assert!(context.sp >= bos as u32);
        ProcessControlBlock{
            pid,
            status: ProcessStatus::Ready,
            stack,
            context,
            file_descriptors,
        }
    }

    pub fn write(&mut self, fid: i32, data: &[u8]) -> Result<usize, FileError> {
        match self.file_descriptors.get(&fid) {
            None => { Err(FileError::InvalidDescriptor) },
            Some(file) => {
                file.write(data).map(|x| {
                    if x.blocked {
                        self.status = ProcessStatus::Blocked;
                    }
                    x.bytes
                })
            },
        }
    }

    pub fn read(&mut self, fid: i32, buffer: &mut [u8]) -> Result<usize, FileError> {
        match self.file_descriptors.get(&fid) {
            None => { Err(FileError::InvalidDescriptor) },
            Some(file) => {
                file.read(buffer).map(|x| {
                    if x.blocked {
                        self.status = ProcessStatus::Blocked;
                    }
                    x.bytes
                })
            },
        }
    }

    pub fn close(&mut self, fid: i32) -> Result<(), FileError> {
        self.file_descriptors.remove(&fid).map(|_| ()).ok_or(FileError::InvalidDescriptor)
    }

}

impl ProcessManager {

    // Create a new process
    pub fn create_process(&mut self, main: unsafe extern fn(), file_descriptors: FidTable) -> PID {
        let pid = self.table.new_key().unwrap();
        let stack = uninit_bytes(DEFAULT_STACK_BYTES);
        let tos = stack.last().unwrap() as *const _;         // last() because the stack grows downwards from higher -> lower addresses
        let pcb = ProcessControlBlock::new(pid, stack, Context::new(main as u32, tos as u32), file_descriptors);
        let process = Rc::new(RefCell::new(pcb));
        self.table.insert(pid, Rc::clone(&process));
        self.scheduler.insert_process(Rc::clone(&process));
        pid
    }

    // Signals sending not implemented, just does SIGKILL regardless of code
    pub fn signal(&mut self, pid: PID, _signal: i32) -> Result<(), String> {
        let x = self.table.remove(&pid).ok_or("PID not found")?;
        let mut borrow = x.borrow_mut();
        borrow.status = ProcessStatus::Terminated;
        self.table.remove(&borrow.pid);
        self.scheduler.remove_process(&x);
        write!(UART0(), "[Killed {}]", borrow.pid).ok();
        Ok(())
    }

    // Forks current process, returns the child PID
    pub fn fork(&mut self, ctx: &Context) -> PID {
        let current = self.scheduler.current_process().unwrap();
        let borrowed = current.borrow();
        let new_pid = self.table.new_key().unwrap();
        let new_stack = borrowed.stack.clone();
        let remapped_sp = adjust_sp(&borrowed.stack, &new_stack, ctx.sp);
        let mut new_ctx = ctx.clone();
        new_ctx.sp = remapped_sp;
        new_ctx.gpr[0] = 0;
        let pcb = ProcessControlBlock::new(new_pid, new_stack, new_ctx, borrowed.file_descriptors.clone());
        let process = Rc::new(RefCell::new(pcb));
        self.table.insert(new_pid, Rc::clone(&process));
        self.scheduler.insert_process(Rc::clone(&process));
        return new_pid
    }

    // Change current process to new executable address
    pub fn exec(&mut self, ctx: &mut Context, address: u32) {
        let current = self.scheduler.current_process().unwrap();
        let borrowed = current.borrow_mut();
        let tos = borrowed.stack.last().unwrap() as *const _;
        *ctx = Context::new(address, tos as u32);
    }

    // Exits current process
    pub fn exit(&mut self, _code: u32) {
        let current = self.scheduler.current_process().unwrap();
        let mut borrowed = current.borrow_mut();
        borrowed.status = ProcessStatus::Exited;
        self.table.remove(&borrowed.pid);
        write!(UART0(), "[{} Exited]", borrowed.pid).ok();
    }

    pub fn current_process(&mut self) -> Option<StrongPcbRef> {
        self.scheduler.current_process()
    }

    pub fn dispatch(&mut self, ctx: &mut Context, src: ScheduleSource) {
        self.scheduler.schedule(src, |prev, mut next| {
            let prev_pid_str = match prev {
                Some(mut x) => {
                    x.context = *ctx;
                    if x.status == ProcessStatus::Executing {   //Only if the previous was in an executing state, e.g. not waiting
                        x.status = ProcessStatus::Ready;
                    }
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
pub fn uninit_bytes(size: usize) -> Vec<u8> {
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
