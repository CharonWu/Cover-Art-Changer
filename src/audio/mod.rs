mod block_type;
mod cover_art;
mod flac;

pub use flac::FlacStream;

pub mod media_center {
    use crate::audio::block_type::FlacBlockType;
    pub use crate::audio::block_type::FlacBlockType::*;
    use crate::audio::cover_art::CoverArt;
    use crate::audio::FlacStream;
    use crate::image::Image::read_image_bytes;
    use crate::tools::ReadFileError;
    use std::path::PathBuf;
    use std::thread::JoinHandle;
    use std::{env, fs, process, thread};

    pub(crate) mod music_config {
        pub const MASK: u8 = 127;
    }

    struct Config {
        command: String,
        audio_files: Vec<String>,
        cover_art: String,
    }

    impl Config {
        fn new() -> Self {
            let args: Vec<String> = env::args().collect();

            assert!(args.len() > 2);

            let mut audio_files: Vec<String> = Vec::new();
            let mut cover_art = String::new();
            match &args[1][..] {
                "-e" => {
                    assert_eq!(args.len(), 3);
                    audio_files = Self::read_audio_files(&args[2]);
                }
                "-r" => {
                    assert_eq!(args.len(), 4);
                    audio_files = Self::read_audio_files(&args[2]);
                    cover_art = String::from(&args[3]);
                }
                _ => {
                    eprintln!("Unknown command!");
                    process::exit(1);
                }
            }

            Config {
                command: args[1].clone(),
                audio_files,
                cover_art,
            }
        }

        fn read_audio_files(path: &String) -> Vec<String> {
            let path_buf = PathBuf::from(path);
            let mut audio_files = Vec::new();
            match path_buf.is_dir() {
                true => {
                    let paths = fs::read_dir(path_buf).unwrap();
                    for p in paths {
                        audio_files.push(String::from(p.unwrap().path().to_str().unwrap()));
                    }
                }
                false => {
                    audio_files.push(path.clone());
                }
            }

            audio_files
        }
    }

    pub fn run() {
        let config = Config::new();

        let mut handles = Vec::new();

        for file in config.audio_files {
            let handle = match &config.command[..] {
                "-e" => export_cover_art(file),
                "-r" => replace_cover_art(file, config.cover_art.clone()),
                _ => {
                    eprintln!("Unknown command!");
                    process::exit(1);
                }
            };
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }
    }

    fn export_cover_art(file: String) -> JoinHandle<()> {
        thread::spawn(move || {
            let music_file = match FlacStream::from_file(&file) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Read file \"{file}\" failed!\nError message: {}", e);
                    process::exit(1);
                }
            };
            match music_file.extract_cover_art() {
                Ok(()) => {
                    println!("{file}: Cover art export successful!");
                }
                Err(e) => {
                    eprintln!("{file}: Cover art export failed\nError message: {e}");
                }
            }
        })
    }

    fn replace_cover_art(file: String, image: String) -> JoinHandle<()> {
        thread::spawn(move || {
            let music_file = match FlacStream::from_file(&file) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Read file \"{file}\" failed!\nError message: {}", e);
                    process::exit(1);
                }
            };
            let image_stream = match read_image_bytes(&image) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Read file \"{file}\" failed!\nError message: {}", e);
                    process::exit(1);
                }
            };
            match music_file.replace_cover_art(&image_stream) {
                Ok(()) => {
                    println!("{file}: Cover art replace successful!");
                }
                Err(e) => {
                    eprintln!("{file}: Cover art replace failed\nError message: {e}");
                }
            }
        })
    }

    pub(crate) trait ImageBlock {
        fn search_block<F>(block_i: usize, stream: &Vec<u8>, f: F) -> usize
        where
            F: Fn(FlacBlockType) -> bool;
        fn start_of_image(stream: &Vec<u8>) -> usize;
        fn get_image_block(pic_block_i: usize, stream: &Vec<u8>)
            -> Result<CoverArt, ReadFileError>;
        fn end_of_image(pic_block_i: usize, stream: &Vec<u8>) -> usize;
    }
}
