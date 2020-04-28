#![allow(non_snake_case)]

use crate::bindings;
use crate::bindings::{PL011_t, PL011_putc, PL011_getc, PL011_can_putc, PL011_can_getc};
use core::fmt::{Write, Error};
use core::result::Result;
use crate::io::descriptor::{FileDescriptor, FileDescriptorBase};
use crate::io::{FileError, IOResult};
use crate::io::error::FileError::UnsupportedOperation;

#[derive(Clone)]
pub struct PL011(*mut PL011_t);

pub fn UART0() -> PL011 {
    unsafe { PL011(bindings::UART0) }
}

pub fn UART1() -> PL011 {
    unsafe { PL011(bindings::UART1) }
}

impl Write for PL011 {

    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        s.as_bytes().iter().for_each(|b| {
            unsafe { PL011_putc(self.0, *b, true) };
        });
        Ok(())
    }
}

pub struct PL011FileDescriptor {
    internal: PL011,
    base: FileDescriptorBase,
    read: bool,
    write: bool,
}

impl PL011FileDescriptor {
    pub fn new(internal: PL011, read: bool, write: bool) -> Self {
        assert!(read || write);
        PL011FileDescriptor {
            internal,
            base: Default::default(),
            read,
            write,
        }
    }
}

impl FileDescriptor for PL011FileDescriptor {

    fn base(&mut self) -> &mut FileDescriptorBase {
        &mut self.base
    }

    fn read(&self, buffer: &mut [u8]) -> Result<IOResult, FileError> {
        if !self.read { return Err(UnsupportedOperation) }
        let mut idx = 0;
        while idx < buffer.len() {
            if unsafe {PL011_can_getc(self.internal.0)} {
                buffer[idx] = unsafe { PL011_getc(self.internal.0, true) };
                idx = idx + 1;
            } else {
                return Ok(IOResult{ bytes: idx, blocked: true })
            }
        };
        Ok(IOResult{ bytes: idx, blocked: false })
    }

    fn write(&self, data: &[u8]) -> Result<IOResult, FileError> {
        if !self.write { return Err(UnsupportedOperation) }
        let mut idx = 0;
        while idx < data.len() {
            if unsafe {PL011_can_putc(self.internal.0)} {
                unsafe { PL011_putc(self.internal.0, data[idx], true) };
                idx = idx + 1;
            } else {
                return Ok(IOResult{ bytes: idx, blocked: true })
            }
        };
        Ok(IOResult{ bytes: idx, blocked: false })
    }

}
