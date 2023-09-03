pub mod Image {
    use crate::tools::ReadFileError;
    use std::fs;

    pub fn read_image_bytes(file_path: &str) -> Result<Vec<u8>, ReadFileError> {
        match fs::read(file_path) {
            Ok(s) => Ok(s),
            Err(e) => Err(ReadFileError::FilePathError(
                "Cannot read image file!".to_string(),
            )),
        }
    }
}
