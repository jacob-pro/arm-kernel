use crate::io::error::FileError;

pub struct IOResult {
    pub bytes: usize,
    pub blocked: bool
}

pub trait FileDescriptor {

    #[allow(unused_variables)]
    fn read(&self, buffer: &mut [u8]) -> Result<IOResult, FileError> {
        Err(FileError::UnsupportedOperation)
    }

    #[allow(unused_variables)]
    fn write(&self, data: &[u8]) -> Result<IOResult, FileError> {
        Err(FileError::UnsupportedOperation)
    }

}
