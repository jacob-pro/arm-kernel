#![allow(non_snake_case)]

use crate::bindings;
use crate::bindings::PL011_t;
use crate::bindings::PL011_putc;
use core::fmt::{Write, Error};
use core::result::Result;

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
            unsafe { PL011_putc( self.0, *b, true ) };
        });
        Ok(())
    }
}

impl PL011 {

    pub fn write_ln(&mut self, s: &str) -> Result<(), Error> {
        self.write_fmt(format_args!("{}\n", s))
    }

}
