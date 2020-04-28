#![allow(non_snake_case)]
#![allow(dead_code)]

pub mod PL011;

mod error;
mod descriptor;

pub use error::FileError;
pub use descriptor::FileDescriptor;
pub use descriptor::IOResult;

pub const STDIN_FILENO: i32 = 0;
pub const STDOUT_FILENO: i32 = 1;
pub const STDERR_FILENO: i32 = 2;
pub const UART1_FILENO: i32 = 3;
