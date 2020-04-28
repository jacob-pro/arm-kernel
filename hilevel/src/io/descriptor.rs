use crate::io::error::FileError;
use alloc::rc::Rc;
use core::cell::RefCell;

pub type StrongFileDescriptorRef = Rc<RefCell<dyn FileDescriptor>>;

pub struct IOResult {
    pub bytes: usize,
    pub blocked: bool
}

#[derive(Default)]
pub struct FileDescriptorBase {

}

impl FileDescriptorBase {
    fn on_interrupt(&mut self) {

    }
}

pub trait FileDescriptor {

    fn base(&mut self) -> &mut FileDescriptorBase;

    fn on_state_change(&mut self) {
        self.base().on_interrupt()
    }

    #[allow(unused_variables)]
    fn read(&self, buffer: &mut [u8]) -> Result<IOResult, FileError> {
        Err(FileError::UnsupportedOperation)
    }

    #[allow(unused_variables)]
    fn write(&self, data: &[u8]) -> Result<IOResult, FileError> {
        Err(FileError::UnsupportedOperation)
    }

}
