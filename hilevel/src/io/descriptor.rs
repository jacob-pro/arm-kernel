use alloc::rc::{Rc, Weak};
use core::cell::RefCell;
use alloc::collections::VecDeque;
use crate::io::tasks::Task;
use alloc::boxed::Box;
use num::range;

pub type StrongFileDescriptorRef = Rc<RefCell<dyn FileDescriptor>>;
pub type WeakFileDescriptorRef = Weak<RefCell<dyn FileDescriptor>>;

pub struct IOResult {
    pub bytes: usize,
    pub blocked: bool
}

#[derive(Default)]
pub struct FileDescriptorBase {
    pending_tasks: VecDeque<Box<dyn Task>>
}

pub enum FileError {
    InvalidDescriptor,
    UnsupportedOperation,
}

pub fn on_file_event(f: &mut dyn FileDescriptor) {
    for _i in range(0, f.base().pending_tasks.len()) {
        let mut popped = f.base().pending_tasks.pop_front().unwrap();
        let result = (*popped).attempt(f);
        if result.is_none() { f.base().pending_tasks.push_back(popped) }
    }
}

pub trait FileDescriptor {

    fn base(&mut self) -> &mut FileDescriptorBase;

    fn add_task(&mut self, task: Box<dyn Task>) {
        self.base().pending_tasks.push_back(task)
    }

    #[allow(unused_variables)]
    fn read(&mut self, buffer: &mut [u8]) -> Result<IOResult, FileError> {
        Err(FileError::UnsupportedOperation)
    }

    #[allow(unused_variables)]
    fn write(&mut self, data: &[u8]) -> Result<IOResult, FileError> {
        Err(FileError::UnsupportedOperation)
    }

}
