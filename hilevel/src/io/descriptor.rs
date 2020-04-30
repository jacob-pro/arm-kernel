use alloc::rc::Rc;
use core::cell::RefCell;
use alloc::collections::VecDeque;
use crate::io::tasks::{ReadTask, WriteTask};
use num::range;
use core::fmt::Debug;

pub type StrongFileDescriptorRef = Rc<RefCell<dyn FileDescriptor>>;

pub struct IOResult {
    pub bytes: usize,
    pub blocked: bool       // Blocked means that there are more bytes still to be read when the file is ready
}

#[derive(Default, Debug)]
pub struct FileDescriptorBase {
    pending_reads: VecDeque<ReadTask>,
    pending_writes: VecDeque<WriteTask>
}

pub enum FileError {
    InvalidDescriptor,
    UnsupportedOperation,
}

// An "abstract class" for different types of files, accessed through the read/write API
pub trait FileDescriptor: Debug {

    fn base(&mut self) -> &mut FileDescriptorBase;

    fn add_pending_read(&mut self, task: ReadTask) {
        self.base().pending_reads.push_back(task)
    }

    fn add_pending_write(&mut self, task: WriteTask) {
        self.base().pending_writes.push_back(task)
    }

    fn notify_pending_readers(&mut self) {
        for _i in range(0, self.base().pending_reads.len()) {
            let mut popped = self.base().pending_reads.pop_front().unwrap();
            let result = popped.attempt(|x| self.read(x) );
            if result.is_none() { self.base().pending_reads.push_back(popped) }
        }
    }

    fn notify_pending_writers(&mut self) {
        for _i in range(0, self.base().pending_writes.len()) {
            let mut popped = self.base().pending_writes.pop_front().unwrap();
            let result = popped.attempt(|x| self.write(x) );
            if result.is_none() { self.base().pending_writes.push_back(popped) }
        }
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
