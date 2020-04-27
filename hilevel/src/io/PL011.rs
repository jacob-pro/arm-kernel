#![allow(non_snake_case)]

use crate::bindings;
use crate::bindings::PL011_t;
use crate::bindings::{PL011_putc, PL011_getc};
use core::fmt::{Write, Error};
use core::result::Result;
use crate::io::descriptor::FileDescriptor;
use crate::io::FileError;

#[derive(Clone)]
pub struct PL011(*mut PL011_t);

pub fn UART0() -> PL011 {
    unsafe { PL011(bindings::UART0) }
}

pub fn UART1() -> PL011 {
    unsafe { PL011(bindings::UART1) }
}

pub fn UART2() -> PL011 {
    unsafe { PL011(bindings::UART2) }
}

pub fn UART3() -> PL011 {
    unsafe { PL011(bindings::UART3) }
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

    fn read(&self, buffer: &mut [u8]) -> Result<usize, FileError> {
        buffer.iter_mut().for_each(|x| {
            *x = unsafe { PL011_getc(self.0, true) };
        });
        Ok(buffer.len())
    }

    fn write(&self, data: &[u8]) -> Result<usize, FileError> {
        data.iter().for_each(|b| {
            unsafe { PL011_putc(self.0, *b, true) };
        });
        Ok(data.len())
    }

}
