use alloc::vec::Vec;
use alloc::string::String;
use alloc::borrow::ToOwned;
use crate::io::error::FileError;

pub trait FileDescriptor {

    fn read(&self, buffer: &mut [u8]) -> Result<usize, FileError> {
        Err(FileError::UnsupportedOperation)
    }

    fn write(&self, data: &[u8]) -> Result<usize, FileError> {
        Err(FileError::UnsupportedOperation)
    }

}
