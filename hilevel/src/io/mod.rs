#![allow(non_snake_case)]
#![allow(dead_code)]

pub mod PL011;
pub mod descriptor;

mod error;
pub use error::FileError;

pub const STDIN_FILENO: i32 = 0;
pub const STDOUT_FILENO: i32 = 1;
pub const STDERR_FILENO: i32 = 2;
pub const UART1_FILENO: i32 = 3;
