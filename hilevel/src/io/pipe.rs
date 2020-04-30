use alloc::collections::VecDeque;
use crate::io::descriptor::{FileDescriptor, FileError, IOResult, FileDescriptorBase, StrongFileDescriptorRef};
use core::cell::RefCell;
use alloc::rc::{Rc, Weak};


const PIPE_BUFFER: usize = 4096;

#[derive(Debug)]
pub struct UnnamedPipe {
    buffer: VecDeque<u8>,
    read_end: Weak<RefCell<PipeReadEnd>>,       // Weak ref to pipe ends to avoid cycle
    write_end: Weak<RefCell<PipeWriteEnd>>,
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

#[derive(Debug)]
pub struct PipeReadEnd {
    pipe: Rc<RefCell<UnnamedPipe>>,     // Keep a strong reference to the internal pipe
    base: FileDescriptorBase,
}

impl PipeReadEnd {
    // When we have read some bytes from the pipe, notify any blocked writers so they can send more
    fn notify_write_end(&mut self) {
        let w = (*self.pipe).borrow_mut().write_end.upgrade();
        w.map(|w| {
            // try_borrow_mut will prevent an infinite Read-Write-Read cycle
            let write_end = w.try_borrow_mut().ok();
            write_end.map(|mut write_end| write_end.notify_pending_writers());
        });
    }
}

#[derive(Debug)]
pub struct PipeWriteEnd {
    pipe: Rc<RefCell<UnnamedPipe>>,
    base: FileDescriptorBase,
}

impl PipeWriteEnd {
    // When we have written bytes to the pipe, notify any blocked readers so they can read them
    fn notify_read_end(&mut self) {
        let r = (*self.pipe).borrow_mut().read_end.upgrade();
        r.map(|r| {
            // try_borrow_mut will prevent an infinite Write-Read-Write cycle, because the read end will have already been borrowed
            let read_end = r.try_borrow_mut().ok();
            read_end.map(|mut read_end| read_end.notify_pending_readers());
        });
    }
}

impl FileDescriptor for PipeReadEnd {

    fn base(&mut self) -> &mut FileDescriptorBase { &mut self.base }

    fn read(&mut self, buffer: &mut [u8]) -> Result<IOResult, FileError> {
        let mut idx = 0;
        while idx < buffer.len() {
            let mut pipe = self.pipe.try_borrow_mut().unwrap();
            if pipe.buffer.is_empty() {
                return Ok(IOResult{ bytes: idx, blocked: true })        // We are blocked, we need to wait for the new writes
            } else {
                buffer[idx] = pipe.buffer.pop_front().unwrap();
                idx = idx + 1;
                drop(pipe);
                self.notify_write_end();  // Writers may be able to give us some more bytes
            }
        };
        Ok(IOResult{ bytes: idx, blocked: false })
    }
}

impl FileDescriptor for PipeWriteEnd {

    fn base(&mut self) -> &mut FileDescriptorBase { &mut self.base }

    fn write(&mut self, data: &[u8]) -> Result<IOResult, FileError> {
        let mut idx = 0;
        while idx < data.len() {
            let mut pipe = self.pipe.borrow_mut();
            if pipe.buffer.len() < PIPE_BUFFER {
                pipe.buffer.push_back(data[idx]);
                idx = idx + 1;
                drop(pipe);
                self.notify_read_end(); // Blocked readers may be able to take away some bytes for us
            } else {
                return Ok(IOResult{ bytes: idx, blocked: true })        // There is nobody left waiting to read any bytes from the buffer
            }
        };
        Ok(IOResult{ bytes: idx, blocked: false })
    }
}
