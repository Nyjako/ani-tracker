use std::fs::{DirEntry, read_dir};
use rayon::prelude::*;
use crate::video::{get_video_length, video_formats};

pub struct DirAnimeEntry {
    pub entry: DirEntry,
    pub length: u64,
}

pub fn scan_directory(path: &str) -> Result<Vec<DirAnimeEntry>, String> {
    let mut output: Vec<DirAnimeEntry> = Vec::new();

    let files = read_dir(path).map_err(|err| err.to_string())?;

    files.enumerate().for_each(|(i, f)| {
        let file = f.unwrap();
        let file_type = file.file_type().unwrap();
        let file_path = file.path();

        if file_type.is_dir() {
            match scan_directory(file_path.to_str().unwrap()) {
                Ok(mut files) => output.append(&mut files),
                Err(err) => eprintln!("{}", err)
            }
            return;
        }

        if let Some(extension) = file_path.extension() {
            if let Some(ext_str) = extension.to_str() {
                if video_formats().contains(&ext_str) {
                    if let Some(file_path_str) = file_path.to_str() {
                        output.push(DirAnimeEntry {
                            entry: file,
                            length: get_video_length(file_path_str).unwrap()
                        });
                    }
                }
            }
        }

    });

    Ok(output)
}