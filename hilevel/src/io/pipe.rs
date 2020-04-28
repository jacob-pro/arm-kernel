use alloc::collections::VecDeque;
use crate::io::descriptor::{FileDescriptor, FileError, IOResult, FileDescriptorBase, StrongFileDescriptorRef};
use core::cell::RefCell;
use alloc::rc::{Rc, Weak};

const PIPE_BUFFER: usize = 4096;

pub struct UnnamedPipe {
    buffer: VecDeque<u8>,
    read_end: Weak<RefCell<PipeReadEnd>>,       // Weak ref to pipe ends to avoid cycle
    write_end: Weak<RefCell<PipeWriteEnd>>,
}

impl UnnamedPipe {

    // When we have read some bytes from the pipe, notify any blocked writers so they can send more
    fn notify_write_end(&mut self) {
        let w = self.write_end.upgrade();
        w.map(|w| {
            w.borrow_mut().notify_pending_writers();
        });
    }

    // When we have written bytes to the pipe, notify any blocked readers so they can read them
    fn notify_read_end(&mut self) {
        let r = self.read_end.upgrade();
        r.map(|r| {
            r.borrow_mut().notify_pending_readers();
        });
    }
}

pub fn new_pipe() -> (StrongFileDescriptorRef, StrongFileDescriptorRef) {

    let pipe = Rc::new(RefCell::new(
        UnnamedPipe {
            buffer: Default::default(),
            read_end: Default::default(),
            write_end: Default::default()
        }));

    let write = Rc::new(RefCell::new(
        PipeWriteEnd{ pipe: Rc::clone(&pipe), base: Default::default() }
    ));
    let read = Rc::new(RefCell::new(
        PipeReadEnd{ pipe: Rc::clone(&pipe), base: Default::default() }
    ));

    let mut l = pipe.borrow_mut();
    l.read_end = Rc::downgrade(&read);
    l.write_end = Rc::downgrade(&write);

    (read, write)
}

pub struct PipeReadEnd {
    pipe: Rc<RefCell<UnnamedPipe>>,     // Keep a strong reference to the internal pipe
    base: FileDescriptorBase,
}

pub struct PipeWriteEnd {
    pipe: Rc<RefCell<UnnamedPipe>>,
    base: FileDescriptorBase,
}

impl FileDescriptor for PipeReadEnd {

    fn base(&mut self) -> &mut FileDescriptorBase { &mut self.base }

    fn read(&mut self, buffer: &mut [u8]) -> Result<IOResult, FileError> {
        let mut pipe = self.pipe.borrow_mut();
        let mut idx = 0;
        while idx < buffer.len() {
            if pipe.buffer.is_empty() {
                pipe.notify_write_end();
                return Ok(IOResult{ bytes: idx, blocked: true })
            } else {
                buffer[idx] = pipe.buffer.pop_front().unwrap();
                idx = idx + 1;
            }
        };
        pipe.notify_write_end();
        Ok(IOResult{ bytes: idx, blocked: false })
    }
}

impl FileDescriptor for PipeWriteEnd {

    fn base(&mut self) -> &mut FileDescriptorBase { &mut self.base }

    fn write(&mut self, data: &[u8]) -> Result<IOResult, FileError> {
        let mut pipe = self.pipe.borrow_mut();
        let mut idx = 0;
        while idx < data.len() {
            if pipe.buffer.len() < PIPE_BUFFER {
                pipe.buffer.push_back(data[idx]);
                idx = idx + 1;
            } else {
                pipe.notify_read_end();
                return Ok(IOResult{ bytes: idx, blocked: true })
            }
        };
        pipe.notify_read_end();
        Ok(IOResult{ bytes: idx, blocked: false })
    }
}
