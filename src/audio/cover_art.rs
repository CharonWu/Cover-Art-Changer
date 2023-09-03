use crate::tools::WriteFileError;
use std::fs::File;
use std::io::{BufWriter, Write};

pub struct CoverArt {
    cover_art_type: String,
    start: usize,
    length: usize,
}

impl CoverArt {
    pub(crate) fn new(cover_art_type: String, start: usize, length: usize) -> Self {
        CoverArt {
            cover_art_type,
            start,
            length,
        }
    }
    pub(crate) fn get_start(&self) -> usize {
        self.start
    }
    pub(crate) fn export_cover_art(
        &self,
        stream: &Vec<u8>,
        file_name: &str,
    ) -> Result<(), WriteFileError> {
        let image_ext = self.cover_art_type.split("/").collect::<Vec<&str>>()[1];
        let file_path = file_name.to_string() + "_cover_art." + image_ext;
        let file = match File::create(file_path) {
            Ok(f) => f,
            Err(_) => {
                return Err(WriteFileError::CreateFileError(
                    "Cannot create file for the exported image!".to_string(),
                ))
            }
        };
        match BufWriter::new(file).write_all(&stream[self.start..(self.start + self.length)]) {
            Ok(()) => Ok(()),
            Err(_) => Err(WriteFileError::CreateFileError(
                "Failed to write image into file!".to_string(),
            )),
        }
    }
}
