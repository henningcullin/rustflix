use std::fs;

use rusqlite::{params, Connection, OptionalExtension};

use crate::{
    database::create_connection,
    directories::{actions::get_all_directories, Directory},
    error::Error,
};

use super::Film;

pub fn get_all_films() -> Result<Vec<Film>, Error> {
    let conn = create_connection()?;

    let mut stmt = conn
        .prepare("SELECT id, file, directory, link, title, release_year, duration, cover_image, synopsis, registered FROM films")?;
    let films: Vec<Film> = stmt
        .query_map([], |row| Film::from_row(row))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(films)
}

pub fn get_film(id: u32) -> Result<Film, Error> {
    let conn = create_connection()?;

    let mut stmt = conn.prepare("SELECT id, file, directory, link, title, release_year, duration, cover_image, synopsis, registered FROM films WHERE id = ?1")?;
    let film = stmt.query_row([id], |row| Ok(Film::from_row(row)?))?;

    Ok(film)
}

pub fn get_files(directory: &Directory) -> Result<Vec<String>, Error> {
    // Video file extensions to look for
    let video_extensions = vec!["mp4", "mkv", "avi", "mov"];

    let mut video_files: Vec<String> = Vec::new();

    if let Ok(entries) = fs::read_dir(&directory.path) {
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
    }

    Ok(video_files)
}

fn add_film(conn: &Connection, file: &String) -> Result<(), Error> {
    conn.execute("INSERT INTO films (file) VALUES (?1)", params![file])?;

    Ok(())
}

pub fn sync_films_with_files() -> Result<(), Error> {
    let conn = create_connection()?;

    let directories = get_all_directories()?;

    for directory in directories {
        let files = get_files(&directory)?;

        for file in files {
            let mut stmt = conn.prepare("SELECT id FROM films WHERE file = ?1")?;

            // Check if the file exists
            let film_exists = stmt
                .query_row([&file], |row| row.get::<_, i32>(0))
                .optional()?;

            if film_exists.is_none() {
                // If the film does not exist, insert a new row
                conn.execute(
                    "INSERT INTO films (file, directory, link, title, release_year, duration, cover_image, synopsis, registered) VALUES (?1, ?2, NULL, NULL, NULL, NULL, NULL, NULL, 0)",
                    params![file, &directory.id],
                )?;
            }
        }
    }

    Ok(())
}
