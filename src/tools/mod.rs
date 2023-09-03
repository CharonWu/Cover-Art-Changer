use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::Utf8Error;

pub mod bytes_converter {
    use crate::tools::ReadFileError;

    pub fn bytes_to_string(bytes: &[u8]) -> Result<&str, ReadFileError> {
        match std::str::from_utf8(bytes) {
            Ok(res) => Ok(res),
            Err(e) => Err(ReadFileError::Utf8Error(e)),
        }
    }

    pub fn usize_to_bytes(value: usize) -> [u8; 4] {
        (value as u32).to_be_bytes()
    }

    pub fn bytes_to_u32(bytes: &[u8]) -> u32 {
        assert_eq!(bytes.len(), 4);

        let mut buffer = [0u8; 4];
        buffer.clone_from_slice(bytes);
        u32::from_be_bytes(buffer)
    }
}

pub fn get_name_and_ext(filepath: &str) -> (Option<&str>, Option<&str>) {
    let path = Path::new(filepath);
    (
        path.file_stem().and_then(OsStr::to_str),
        path.extension().and_then(OsStr::to_str),
    )
}

pub enum ReadFileError {
    Utf8Error(Utf8Error),
    FilePathError(String),
}

impl Display for ReadFileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadFileError::Utf8Error(e) => write!(f, "{}", e),
            ReadFileError::FilePathError(e) => write!(f, "{}", e),
        }
    }
}

pub enum WriteFileError {
    CreateFileError(String),
    WriteError(String),
}

impl Display for WriteFileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteFileError::CreateFileError(e) => write!(f, "{}", e),
            WriteFileError::WriteError(e) => write!(f, "{}", e),
        }
    }
}
