use std::fs;

use crate::database::create_connection;

use super::Film;

pub fn get_all_films() -> Result<Vec<Film>, rusqlite::Error> {
    let conn = create_connection()?;

    let mut stmt = conn.prepare("SELECT id, title, file, link FROM films")?;
    let films: Vec<Film> = stmt
        .query_map([], |row| {
            Ok(Film {
                id: row.get(0)?,
                title: row.get(1)?,
                file: row.get(2)?,
                link: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(films)
}

pub fn check_for_new_films() -> Result<(), rusqlite::Error> {
    let conn = create_connection()?;

    // Video file extensions to look for
    let video_extensions = vec!["mp4", "mkv", "avi", "mov"];

    let mut stmt = conn.prepare("SELECT path FROM directories")?;
    let directories: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    // Vector to store all video files found
    let mut all_videos: Vec<String> = Vec::new();

    for dir in directories {
        println!("{dir:?}");
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(extension) = path.extension() {
                            if let Some(ext) = extension.to_str() {
                                if video_extensions.contains(&ext) {
                                    if let Some(path_str) = path.to_str() {
                                        all_videos.push(path_str.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            println!("Could not read directory: {dir:?}");
        }
    }

    // Print all found video files
    for video in all_videos {
        println!("{}", video);
    }

    Ok(())
}
