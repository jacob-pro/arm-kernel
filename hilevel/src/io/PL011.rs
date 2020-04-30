#![allow(non_snake_case)]

use crate::bindings;
use crate::bindings::{PL011_t, PL011_putc, PL011_getc};
use core::fmt::{Write, Error};
use core::result::Result;
use crate::io::descriptor::{FileDescriptor, FileDescriptorBase, IOResult, FileError};
use alloc::collections::VecDeque;

const KEYBOARD_BUFFER: usize = 4096;

#[derive(Clone, Debug)]
pub struct PL011(*mut PL011_t);

pub fn UART0() -> PL011 {
    unsafe { PL011(bindings::UART0) }
}

pub fn UART1() -> PL011 {
    unsafe { PL011(bindings::UART1) }
}

impl PL011 {
    fn putc(&self, byte: u8, blocking: bool) {
        unsafe { PL011_putc(self.0, byte, blocking) };
    }
    fn getc(&self, blocking: bool) -> u8 {
        unsafe { PL011_getc(self.0, blocking) }
    }
}

impl Write for PL011 {

    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        s.as_bytes().iter().for_each(|b| {
            self.putc(*b, true);
        });
        Ok(())
    }
}

#[derive(Debug)]
pub struct PL011FileDescriptor {
    internal: PL011,
    base: FileDescriptorBase,
    read: bool,
    write: bool,
    read_buffer: VecDeque<u8>,
}

impl PL011FileDescriptor {

    pub fn new(internal: PL011, read: bool, write: bool) -> Self {
        assert!(read || write);
        PL011FileDescriptor {
            internal,
            base: Default::default(),
            read,
            write,
            read_buffer: Default::default()
        }
    }

    // Add chars to the input buffer, then notify any blocked readers
    pub fn on_interrupt(&mut self) {
        let char = self.internal.getc(true);
        if self.read_buffer.len() < KEYBOARD_BUFFER {
            self.read_buffer.push_back(char);
            self.notify_pending_readers();
        }
    }

}

impl FileDescriptor for PL011FileDescriptor {

    fn base(&mut self) -> &mut FileDescriptorBase {&mut self.base}

    // This will return blocked until input is available
    fn read(&mut self, buffer: &mut [u8]) -> Result<IOResult, FileError> {
        if !self.read { return Err(FileError::UnsupportedOperation) }
        let mut idx = 0;
        while idx < buffer.len() {
            if self.read_buffer.is_empty() {
                return Ok(IOResult{ bytes: idx, blocked: true })
            } else {
                buffer[idx] = self.read_buffer.pop_front().unwrap();
                idx = idx + 1;
            }
        };
        Ok(IOResult{ bytes: idx, blocked: false })
    }

    fn write(&mut self, data: &[u8]) -> Result<IOResult, FileError> {
        if !self.write { return Err(FileError::UnsupportedOperation) }
        data.iter().for_each(|b| {
            self.internal.putc(*b, true);
        });
        Ok(IOResult{ bytes: data.len(), blocked: false })
    }

}
