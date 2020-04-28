#![allow(non_snake_case)]
#![allow(dead_code)]

pub mod PL011;

mod error;
mod descriptor;

pub use error::FileError;
pub use descriptor::FileDescriptor;
pub use descriptor::IOResult;
pub use descriptor::StrongFileDescriptorRef;
use crate::process::FidTable;
use alloc::rc::Rc;
use crate::io::PL011::{UART0, UART1};

pub const STDIN_FILENO: i32 = 0;
pub const STDOUT_FILENO: i32 = 1;
pub const STDERR_FILENO: i32 = 2;
pub const UART1_FILENO: i32 = 3;

pub struct IoManager {
    uart0_ro: StrongFileDescriptorRef,
    uart0_wo: StrongFileDescriptorRef,
    uart1_rw: StrongFileDescriptorRef,
}

impl IoManager {

    pub fn default_files(&self) -> FidTable {
        let mut table = FidTable::default();
        #[cfg(not(test))]
            {
                table.insert(STDIN_FILENO, Rc::clone(&self.uart0_ro));
                table.insert(STDOUT_FILENO, Rc::clone(&self.uart0_wo));
                table.insert(STDERR_FILENO, Rc::clone(&self.uart0_wo));
                table.insert(UART1_FILENO, Rc::clone(&self.uart1_rw));
            }
        table
    }

}

impl Default for IoManager {

    fn default() -> Self {
        IoManager {
            uart0_ro: Rc::new(UART0()),
            uart0_wo: Rc::new(UART0()),
            uart1_rw: Rc::new(UART1()),
        }
    }

}
