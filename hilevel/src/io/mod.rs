#![allow(non_snake_case)]
#![allow(dead_code)]

pub mod PL011;
pub mod tasks;
pub mod descriptor;
pub mod pipe;

use crate::process::FidTable;
use alloc::rc::Rc;
use crate::io::PL011::{UART0, UART1, PL011FileDescriptor};
use core::cell::RefCell;
use crate::io::descriptor::StrongFileDescriptorRef;

pub const STDIN_FILENO: i32 = 0;
pub const STDOUT_FILENO: i32 = 1;
pub const STDERR_FILENO: i32 = 2;
pub const UART1_FILENO: i32 = 3;

pub struct IoManager {
    pub uart0_ro: Rc<RefCell<PL011FileDescriptor>>,
    pub uart0_wo: Rc<RefCell<PL011FileDescriptor>>,
    pub uart1_rw: Rc<RefCell<PL011FileDescriptor>>,
}

impl IoManager {

    pub fn default_files(&self) -> FidTable {
        let mut table = FidTable::default();
        #[cfg(not(test))]
            {
                table.insert(STDIN_FILENO, Rc::clone(&self.uart0_ro) as StrongFileDescriptorRef);
                table.insert(STDOUT_FILENO, Rc::clone(&self.uart0_wo) as StrongFileDescriptorRef);
                table.insert(STDERR_FILENO, Rc::clone(&self.uart0_wo) as StrongFileDescriptorRef);
                table.insert(UART1_FILENO, Rc::clone(&self.uart1_rw) as StrongFileDescriptorRef);
            }
        table
    }

}

impl Default for IoManager {

    fn default() -> Self {
        IoManager {
            uart0_ro: Rc::new(RefCell::new(PL011FileDescriptor::new(UART0(), true, false))),
            uart0_wo: Rc::new(RefCell::new(PL011FileDescriptor::new(UART0(), false, true))),
            uart1_rw: Rc::new(RefCell::new(PL011FileDescriptor::new(UART1(), true, true))),
        }
    }

}
