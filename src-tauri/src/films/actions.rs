use std::{collections::HashMap, fs};

use serde::Serialize;

use crate::{
    database::create_connection,
    directories::{actions::get_all_directories, Directory},
};

use super::Film;

pub fn get_all_films() -> Result<Vec<Film>, rusqlite::Error> {
    let conn = create_connection()?;

    let mut stmt = conn
        .prepare("SELECT id, file, link, title, release_year, duration, cover_image FROM films")?;
    let films: Vec<Film> = stmt
        .query_map([], |row| Film::from_row(row))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(films)
}

#[derive(Serialize)]
struct VideoFiles {
    files_by_directory: HashMap<u32, Vec<String>>,
}

pub fn check_for_new_films() -> Result<(), rusqlite::Error> {
    // Video file extensions to look for
    let video_extensions = vec!["mp4", "mkv", "avi", "mov"];

    let directories: Vec<Directory> = get_all_directories()?;

    let mut video_files_by_directory: HashMap<u32, Vec<String>> = HashMap::new();

    for dir in directories {
        if let Ok(entries) = fs::read_dir(&dir.path) {
            let mut video_files: Vec<String> = Vec::new();

            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(extension) = path.extension() {
                            if let Some(ext) = extension.to_str() {
                                if video_extensions.contains(&ext) {
                                    if let Some(path_str) = path.to_str() {
                                        video_files.push(path_str.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            video_files_by_directory.insert(dir.id.clone(), video_files);
        } else {
            println!("Could not read directory: {:?}", dir);
        }
    }

    // Serialize the HashMap to JSON
    let video_files = VideoFiles {
        files_by_directory: video_files_by_directory,
    };

    let json = serde_json::to_string(&video_files).map_err(|e| e.to_string());

    match json {
        Ok(msg) => println!("{msg}"),
        Err(msg) => println!("{msg}"),
    }

    Ok(())
}
