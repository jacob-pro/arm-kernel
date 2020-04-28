use crate::process::{WeakPcbRef, StrongPcbRef};
use alloc::rc::Rc;
use core::slice;
use crate::io::descriptor::FileDescriptor;

pub struct TaskBase {
    process: WeakPcbRef,
    completed: usize,
    length: usize,
}

pub struct ReadTask {
    base: TaskBase,
    destination: *mut u8,
}

pub struct WriteTask {
    base: TaskBase,
    source: *const u8,
}

pub trait Task {
    // This task can keep being attempted until a result is given
    fn attempt(&mut self, file: &mut dyn FileDescriptor) -> Option<u32>;
}

impl ReadTask {
    pub fn new(process: &StrongPcbRef, destination: *mut u8, length: usize) -> Self {
        ReadTask{ base: TaskBase {
            process: Rc::downgrade(process),
            completed: 0,
            length,
        }, destination }
    }
}

impl WriteTask {
    pub fn new(process: &StrongPcbRef, source: *const u8, length: usize) -> Self {
        WriteTask{ base: TaskBase {
            process: Rc::downgrade(process),
            completed: 0,
            length,
        }, source }
    }
}

impl Task for ReadTask {

    fn attempt(&mut self, file: &mut dyn FileDescriptor) -> Option<u32> {
        let process = self.base.process.upgrade();
        // If the process is gone, then the task is complete
        process.map_or(Some(self.base.completed as u32), |x| {
            let mut borrow = (*x).borrow_mut();
            let slice: &mut [u8] = unsafe {
                let todo = self.base.length - self.base.completed;
                let start_from = self.destination.offset(self.base.completed as isize);
                slice::from_raw_parts_mut(start_from, todo)
            };
            return match file.read(slice) {
                Ok(x) => {
                    self.base.completed = self.base.completed + x.bytes;
                    if x.blocked {
                        borrow.set_blocked();
                        None
                    } else {
                        borrow.set_unblocked(self.base.completed as u32);
                        Some(self.base.completed as u32)
                    }
                },
                Err(_e) => {
                    let error: i32 = -1;
                    borrow.set_unblocked(error as u32);
                    Some(error as u32)        // There was an error, we cannot continue
                },
            }
        })
    }
}

impl Task for WriteTask {

    fn attempt(&mut self, file: &mut dyn FileDescriptor) -> Option<u32> {
        let process = self.base.process.upgrade();
        // If the process is gone, then the task is complete
        process.map_or(Some(self.base.completed as u32), |x| {
            let mut borrow = (*x).borrow_mut();
            let slice: &[u8] = unsafe {
                let todo = self.base.length - self.base.completed;
                let start_from = self.source.offset(self.base.completed as isize);
                slice::from_raw_parts(start_from, todo)
            };
            return match file.write(slice) {
                Ok(x) => {
                    self.base.completed = self.base.completed + x.bytes;
                    if x.blocked {
                        borrow.set_blocked();
                        None
                    } else {
                        borrow.set_unblocked(self.base.completed as u32);
                        Some(self.base.completed as u32)
                    }
                },
                Err(_e) => {
                    let error: i32 = -1;
                    borrow.set_unblocked(error as u32);
                    Some(error as u32)        // There was an error, we cannot continue
                },
            }
        })
    }
}