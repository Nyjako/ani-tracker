use walkdir::{WalkDir, DirEntry};
use crate::video::{get_video_length, video_formats};

const MAX_OPEN: usize = 10;

pub struct DirAnimeEntry {
    pub entry: DirEntry,
    pub length: u64,
}

pub fn scan_directory(path: &str) -> Vec<DirAnimeEntry> {
    let mut output: Vec<DirAnimeEntry> = Vec::new();

    for entry in WalkDir::new(path).max_open(MAX_OPEN) {
        match entry {
            Ok(v) => {
                let supported_video = match v.file_name().to_str() {
                    Some(v) => match v.split('.').last() {
                        Some(v) => {
                            video_formats().contains(&v)
                        },
                        None => continue
                    },
                    None => {
                        println!("Error while getting filename");
                        continue;
                    }
                };

                if !supported_video {
                    println!("\"{:?}\" is not supported yet.", v.file_name());
                    continue;
                }

                let path_str = match v.path().to_str() {
                    Some(v) => v,
                    None => {
                        println!("Error while getting path");
                        continue;
                    }
                };

                let length = match get_video_length(path_str) {
                    Ok(v) => v,
                    Err(e) => {
                        println!("Error while getting video length: {}", e);
                        continue;
                    }
                };

                output.push(DirAnimeEntry {
                    entry: v,
                    length: length,
                });
            },
            Err(e) => println!("Error while scanning directory: {}", e),
        }
    }

    return output;
}