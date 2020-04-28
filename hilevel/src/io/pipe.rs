use alloc::collections::VecDeque;
use crate::io::descriptor::{FileDescriptor, FileError, IOResult, FileDescriptorBase};
use core::cell::RefCell;
use alloc::rc::{Rc, Weak};

const PIPE_BUFFER: usize = 4096;

pub struct UnnamedPipe {
    buffer: VecDeque<u8>,
    read_end: Weak<RefCell<PipeReadEnd>>,
    write_end: Weak<RefCell<PipeWriteEnd>>,
}

pub fn new_pipe() -> (PipeReadEnd, PipeWriteEnd) {
    unimplemented!()
}

pub struct PipeReadEnd {
    pipe: Rc<RefCell<UnnamedPipe>>,
}

pub struct PipeWriteEnd {
    pipe: Rc<RefCell<UnnamedPipe>>,
}

impl FileDescriptor for PipeReadEnd {

    fn base(&mut self) -> &mut FileDescriptorBase {
        unimplemented!()
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<IOResult, FileError> {
        let mut pipe = self.pipe.borrow_mut();
        let mut idx = 0;
        while idx < buffer.len() {
            if pipe.buffer.is_empty() {
                return Ok(IOResult{ bytes: idx, blocked: true })
            } else {
                buffer[idx] = pipe.buffer.pop_front().unwrap();
                idx = idx + 1;
            }
        };
        Ok(IOResult{ bytes: idx, blocked: false })
    }
}

impl FileDescriptor for PipeWriteEnd {

    fn base(&mut self) -> &mut FileDescriptorBase {
        unimplemented!()
    }

    fn write(&mut self, data: &[u8]) -> Result<IOResult, FileError> {
        let mut pipe = self.pipe.borrow_mut();
        let mut idx = 0;
        while idx < data.len() {
            if pipe.buffer.len() < PIPE_BUFFER {
                pipe.buffer.push_back(data[idx]);
                idx = idx + 1;
            } else {
                return Ok(IOResult{ bytes: idx, blocked: true })
            }
        };
        Ok(IOResult{ bytes: idx, blocked: false })
    }
}