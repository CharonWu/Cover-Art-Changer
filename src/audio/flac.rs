use crate::audio::block_type::FlacBlockType;
use crate::audio::cover_art::CoverArt;
use crate::audio::media_center::*;
use crate::tools::bytes_converter::{bytes_to_string, bytes_to_u32, usize_to_bytes};
use crate::tools::{get_name_and_ext, ReadFileError, WriteFileError};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::{fs, process};

pub struct FlacStream<'a> {
    file_name: &'a str,
    format: &'a str,
    stream: Vec<u8>,
    image: CoverArt,
    image_block_start: usize,
    image_block_end: usize,
}

impl<'a> FlacStream<'a> {
    pub fn from_file(filepath: &'a str) -> Result<FlacStream, ReadFileError> {
        let (file_name, ext) = match get_name_and_ext(filepath) {
            (Some(n), Some(t)) => (n, t),
            (Some(_), None) => {
                return Err(ReadFileError::FilePathError(
                    "Unknown file extension!".to_string(),
                ));
            }
            (None, Some(_)) => {
                return Err(ReadFileError::FilePathError(
                    "No file name provided!".to_string(),
                ));
            }
            _ => {
                return Err(ReadFileError::FilePathError("Invalid path!".to_string()));
            }
        };
        let stream = match fs::read(filepath) {
            Ok(s) => s,
            Err(e) => {
                return Err(ReadFileError::FilePathError(
                    "Cannot read file!".to_string(),
                ));
            }
        };
        let image_block_start = FlacStream::start_of_image(&stream);
        let image = FlacStream::get_image_block(image_block_start, &stream)?;
        let image_block_end = FlacStream::end_of_image(image_block_start, &stream);
        Ok(FlacStream {
            file_name,
            format: ext,
            stream,
            image_block_start,
            image,
            image_block_end,
        })
    }

    pub fn extract_cover_art(&self) -> Result<(), WriteFileError> {
        self.image.export_cover_art(&self.stream, self.file_name)?;
        Ok(())
    }

    pub fn replace_cover_art(&self, image_bytes: &Vec<u8>) -> Result<(), WriteFileError> {
        let file_path = self.file_name.to_string() + "." + self.format;
        let file = match File::create(file_path) {
            Ok(f) => f,
            Err(_) => {
                return Err(WriteFileError::CreateFileError(
                    "Cannot create file with replaced cover art!".to_string(),
                ))
            }
        };
        let mut buf_writer = BufWriter::new(file);
        //write data before pic block
        match buf_writer.write_all(&self.stream[..self.image_block_start]) {
            Ok(_) => (),
            Err(_) => {
                return Err(WriteFileError::WriteError(
                    "Failed to write data before cover art block!".to_string(),
                ))
            }
        };

        //write pic block header
        match buf_writer
            .write_all(&self.stream[self.image_block_start..(self.image_block_start + 1)])
        {
            Ok(_) => (),
            Err(_) => {
                return Err(WriteFileError::WriteError(
                    "Failed to write picture block header".to_string(),
                ))
            }
        };

        let new_block_length =
            usize_to_bytes(image_bytes.len() + self.image.get_start() - self.image_block_start - 4);
        match buf_writer.write_all(&new_block_length[1..]) {
            Ok(_) => (),
            Err(_) => {
                return Err(WriteFileError::WriteError(
                    "Failed to write new length of picture block".to_string(),
                ))
            }
        };

        //write data from pic block data to length of cover art
        match buf_writer
            .write_all(&self.stream[(self.image_block_start + 4)..(self.image.get_start() - 4)])
        {
            Ok(_) => (),
            Err(_) => {
                return Err(WriteFileError::WriteError(
                    "Failed to write data from pic block data to length of cover art".to_string(),
                ))
            }
        };

        //write the length of cover art
        match buf_writer.write_all(&usize_to_bytes(image_bytes.len())) {
            Ok(_) => (),
            Err(_) => {
                return Err(WriteFileError::WriteError(
                    "Failed to write length of new cover art!".to_string(),
                ))
            }
        };

        //write cover art data
        match buf_writer.write_all(&image_bytes[..]) {
            Ok(_) => (),
            Err(_) => {
                return Err(WriteFileError::WriteError(
                    "Failed to write new cover art!".to_string(),
                ))
            }
        };

        //write data after cover art
        match buf_writer.write_all(&self.stream[self.image_block_end..self.stream.len()]) {
            Ok(_) => Ok(()),
            Err(_) => {
                return Err(WriteFileError::WriteError(
                    "Failed to write data after cover art block!".to_string(),
                ))
            }
        }
    }
}

impl<'a> ImageBlock for FlacStream<'a> {
    fn search_block<F>(block_i: usize, stream: &Vec<u8>, f: F) -> usize
    where
        F: Fn(FlacBlockType) -> bool,
    {
        let mut block_index = block_i;
        loop {
            let (last_meta_data_block, block_type) = match stream.get(block_index) {
                Some(byte) => {
                    let is_last = if byte >> 7 == 1 { true } else { false };
                    let b_type = match byte & music_config::MASK {
                        0 => StreamInfo,
                        1 => Padding,
                        2 => Application,
                        3 => Seekable,
                        4 => VorbisComment,
                        5 => CueSheet,
                        6 => Picture,
                        7..=126 => Reserved,
                        _ => Invalid,
                    };
                    (is_last, b_type)
                }
                None => panic!("cannot read meta data header!"),
            };
            if f(block_type) {
                break;
            }
            if last_meta_data_block {
                eprintln!("Didn't find picture block!");
                process::exit(1);
            }

            let mut length_buffer = [0u8; 4];
            length_buffer.clone_from_slice(&stream[(block_index)..(block_index + 4)]);
            length_buffer[0] = 0;

            let block_length = u32::from_be_bytes(length_buffer);
            block_index += 4 + block_length as usize;
        }

        block_index
    }

    fn start_of_image(stream: &Vec<u8>) -> usize {
        let block_index: usize = 4;

        FlacStream::search_block(block_index, stream, |t| t == Picture)
    }

    fn get_image_block(pic_block_i: usize, stream: &Vec<u8>) -> Result<CoverArt, ReadFileError> {
        let mut pic_block_i = pic_block_i + 4;

        let metadata_length = bytes_to_u32(&stream[pic_block_i..(pic_block_i + 4)]);

        pic_block_i += 4;
        let mime_length = bytes_to_u32(&stream[pic_block_i..(pic_block_i + 4)]);

        pic_block_i += 4;
        let mime_string =
            bytes_to_string(&stream[pic_block_i..(pic_block_i + mime_length as usize)])?;

        pic_block_i += mime_length as usize;
        let description_length = bytes_to_u32(&stream[pic_block_i..(pic_block_i + 4)]);

        pic_block_i += 4;
        let pic_description =
            bytes_to_string(&stream[pic_block_i..(pic_block_i + description_length as usize)])?;

        pic_block_i += description_length as usize;
        let pic_width = bytes_to_u32(&stream[pic_block_i..(pic_block_i + 4)]);

        pic_block_i += 4;
        let pic_height = bytes_to_u32(&stream[pic_block_i..(pic_block_i + 4)]);

        pic_block_i += 4;
        let color_depth = bytes_to_u32(&stream[pic_block_i..(pic_block_i + 4)]);

        pic_block_i += 4;
        let color_index = bytes_to_u32(&stream[pic_block_i..(pic_block_i + 4)]);

        pic_block_i += 4;
        let pic_data_length = bytes_to_u32(&stream[pic_block_i..(pic_block_i + 4)]);

        pic_block_i += 4;
        Ok(CoverArt::new(
            mime_string.to_string(),
            pic_block_i,
            pic_data_length as usize,
        ))
    }

    fn end_of_image(pic_block_i: usize, stream: &Vec<u8>) -> usize {
        FlacStream::search_block(pic_block_i, stream, |t| t != Picture)
    }
}
