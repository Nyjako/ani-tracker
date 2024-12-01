extern crate anitomy;

use std::borrow::BorrowMut;
use std::fs::{DirEntry, read_dir};
use anitomy::{Anitomy, ElementCategory, Elements};
use sqlx::{Pool, Sqlite};
use rayon::prelude::*;
use std::time::Duration;
use indicatif::ProgressBar;

use crate::video::{get_video_length, video_formats};
use crate::db;

pub struct DirAnimeEntry {
    pub entry: DirEntry,
    pub length: u64,
    pub name: Option<String>,
    pub episode: Option<i64>
}

pub async fn full_scan(db: Pool<Sqlite>) -> Result<Vec<DirAnimeEntry>, String> {
    let mut conn = match db.acquire().await {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Failed to acquire database connection: {}", err);
            std::process::exit(1);
        }
    };

    let mut output: Vec<DirAnimeEntry> = Vec::new();

    let bar = ProgressBar::new_spinner().with_message("🔎 Scanning files");
    bar.enable_steady_tick(Duration::from_millis(100));

    match sqlx::query_as!(
        db::Directories,
        "SELECT * FROM directories"
    ).fetch_all(&mut *conn).await {
        Ok(dirs) => {
            for dir in dirs {
                match scan_directory(&dir.path.as_str()) {
                    Ok(mut scan_result) => output.append(&mut scan_result),
                    Err(err) => eprintln!("{}", err),
                }
            }
        },
        Err(err) => eprintln!("{}", err),
    }

    bar.finish();

    Ok(output)
}

pub async fn async_scan_directory(path: &str) -> Result<Vec<DirAnimeEntry>, String> {
    // TODO: Rewrite this as actual async function this is just hack for now
    scan_directory(path)
}

pub fn scan_directory(path: &str) -> Result<Vec<DirAnimeEntry>, String> {
    let mut output: Vec<DirAnimeEntry> = Vec::new();
    let mut anitomy = Anitomy::new();

    let files = read_dir(path).map_err(|err| err.to_string())?;

    files.enumerate().for_each(|(_i, f)| {
        let file = f.unwrap();
        let file_type = file.file_type().unwrap();
        let file_path = file.path();
        let filename = file.file_name();

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

                        let filename = filename.to_str().unwrap();
                        match anitomy.parse(filename) {
                            Ok(elements) => {
                                let name = elements.get(ElementCategory::AnimeTitle).map(|s| s.to_owned());
                                let episode = elements.get(ElementCategory::EpisodeNumber).map(|s| s.to_owned())
                                .map(|f| {
                                    match f.parse::<i64>() {
                                        Ok(val) => val,
                                        Err(err) => {
                                            println!("Failed to parse string to i64: {}", err);
                                            -1
                                        }
                                    }
                                });
                                output.push(DirAnimeEntry {
                                    entry: file,
                                    length: get_video_length(file_path_str).unwrap(),
                                    name,
                                    episode,
                                });
                            },
                            Err(_err) => eprintln!("Failed to get details from filename"),
                        }
                        
                    }
                }
            }
        }

    });

    Ok(output)
}