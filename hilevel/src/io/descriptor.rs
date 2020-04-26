use alloc::vec::Vec;
use alloc::string::String;
use alloc::borrow::ToOwned;

pub trait FileDescriptor {

    fn read(&self, buffer: &mut [u8]) -> Result<usize, String> {
        Err("Operation not supported".to_owned())
    }

    fn write(&self, data: &[u8]) -> Result<usize, String> {
        Err("Operation not supported".to_owned())
    }

}
