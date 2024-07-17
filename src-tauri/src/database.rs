use std::{fs, path::PathBuf};

use dirs::data_local_dir;
use rusqlite::Connection;

fn get_database_path() -> PathBuf {
    let mut path = data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("Rustflix");
    if !path.exists() {
        fs::create_dir_all(&path).expect("Failed to create appdata directory");
    }

    path.push("database.sqlite");
    path
}

pub fn create_connection() -> Result<Connection, rusqlite::Error> {
    let db_path = get_database_path();
    Connection::open(db_path)
}

pub fn initialize_database() -> Result<(), rusqlite::Error> {
    let conn = create_connection()?;

    // create films table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS films (
            id    INTEGER PRIMARY KEY,
            title TEXT,
            file  TEXT NOT NULL,
            link  TEXT
            )",
        [],
    )?;

    // create directories table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS directories (
        id   INTEGER PRIMARY KEY,
        path TEXT NOT NULL
        )",
        [],
    )?;

    Ok(())
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
            println!("Could not read directory: {}", dir);
        }
    }

    // Print all found video files
    for video in all_videos {
        println!("{}", video);
    }

    Ok(())
}
