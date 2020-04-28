#![allow(non_snake_case)]

use crate::bindings;
use crate::bindings::PL011_t;
use crate::bindings::{PL011_putc, PL011_getc, PL011_can_putc, PL011_can_getc};
use core::fmt::{Write, Error};
use core::result::Result;
use crate::io::descriptor::FileDescriptor;
use crate::io::{FileError, IOResult};

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

impl FileDescriptor for PL011 {

    fn read(&self, buffer: &mut [u8]) -> Result<IOResult, FileError> {
        let mut idx = 0;
        while idx < buffer.len() {
            if unsafe {PL011_can_getc(self.0)} {
                buffer[idx] = unsafe { PL011_getc(self.0, true) };
                idx = idx + 1;
            } else {
                return Ok(IOResult{ bytes: idx, blocked: true })
            }
        };
        Ok(IOResult{ bytes: idx, blocked: false })
    }

    fn write(&self, data: &[u8]) -> Result<IOResult, FileError> {
        let mut idx = 0;
        while idx < data.len() {
            if unsafe {PL011_can_putc(self.0)} {
                unsafe { PL011_putc(self.0, data[idx], true) };
                idx = idx + 1;
            } else {
                return Ok(IOResult{ bytes: idx, blocked: true })
            }
        };
        Ok(IOResult{ bytes: idx, blocked: false })
    }

}
