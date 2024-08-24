use std::fs;

use rusqlite::{params, Connection, OptionalExtension};

use crate::{
    database::create_connection,
    directories::{actions::get_all_directories, Directory},
    error::AppError,
};

use super::Film;

pub fn get_all_films() -> Result<Vec<Film>, AppError> {
    let conn = create_connection()?;

    let mut stmt = conn.prepare(r#"--sql
        SELECT 
            f.id as film_id, f.file, f.directory, f.imdb_id, f.title, 
            f.release_date, f.plot, f.run_time, f.color, f.rating, 
            f.cover_image, f.has_watched, f.left_off_point, f.registered,
            GROUP_CONCAT(DISTINCT g.id || ':' || g.path) as genres,
            GROUP_CONCAT(DISTINCT l.id || ':' || l.name) as languages,
            GROUP_CONCAT(DISTINCT k.keyword) as keywords,
            GROUP_CONCAT(DISTINCT p.id || ':' || p.imdb_id || ':' || p.avatar || ':' || p.age || ':' || p.gender || ':' || p.birthplace) as directors,
            GROUP_CONCAT(DISTINCT c.id || ':' || c.description || ':' || ap.id || ':' || ap.imdb_id || ':' || ap.avatar || ':' || ap.age || ':' || ap.gender || ':' || ap.birthplace) as stars
        FROM films f
        LEFT JOIN film_genres fg ON f.id = fg.film_id
        LEFT JOIN genres g ON fg.genre_id = g.id
        LEFT JOIN film_languages fl ON f.id = fl.film_id
        LEFT JOIN languages l ON fl.language_id = l.id
        LEFT JOIN film_keywords k ON f.id = k.film_id
        LEFT JOIN film_directors fd ON f.id = fd.film_id
        LEFT JOIN persons p ON fd.person_id = p.id
        LEFT JOIN film_characters fc ON f.id = fc.film_id
        LEFT JOIN characters c ON fc.character_id = c.id
        LEFT JOIN persons ap ON c.actor = ap.id
        GROUP BY f.id
    "#)?;

    let films = stmt
        .query_map([], |row| Film::from_row(row))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(films)
}

pub fn get_film(id: u32) -> Result<Film, AppError> {
    let conn = create_connection()?;

    let mut stmt = conn.prepare("--sql
        SELECT 
            f.id as film_id, f.file, f.directory, f.imdb_id, f.title, 
            f.release_date, f.plot, f.run_time, f.color, f.rating, 
            f.cover_image, f.has_watched, f.left_off_point, f.registered,
            GROUP_CONCAT(DISTINCT g.id || ':' || g.path) as genres,
            GROUP_CONCAT(DISTINCT l.id || ':' || l.name) as languages,
            GROUP_CONCAT(DISTINCT k.keyword) as keywords,
            GROUP_CONCAT(DISTINCT p.id || ':' || p.imdb_id || ':' || p.avatar || ':' || p.age || ':' || p.gender || ':' || p.birthplace) as directors,
            GROUP_CONCAT(DISTINCT c.id || ':' || c.description || ':' || ap.id || ':' || ap.imdb_id || ':' || ap.avatar || ':' || ap.age || ':' || ap.gender || ':' || ap.birthplace) as stars
        FROM films f
        LEFT JOIN film_genres fg ON f.id = fg.film_id
        LEFT JOIN genres g ON fg.genre_id = g.id
        LEFT JOIN film_languages fl ON f.id = fl.film_id
        LEFT JOIN languages l ON fl.language_id = l.id
        LEFT JOIN film_keywords k ON f.id = k.film_id
        LEFT JOIN film_directors fd ON f.id = fd.film_id
        LEFT JOIN persons p ON fd.person_id = p.id
        LEFT JOIN film_characters fc ON f.id = fc.film_id
        LEFT JOIN characters c ON fc.character_id = c.id
        LEFT JOIN persons ap ON c.actor = ap.id
            WHERE f.id = ?1")?;
    let film = stmt.query_row([id], |row| Ok(Film::from_row(row)?))?;

    Ok(film)
}

pub fn get_files(directory: &Directory) -> Result<Vec<String>, AppError> {
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

fn add_film(conn: &Connection, file: &String) -> Result<(), AppError> {
    conn.execute("INSERT INTO films (file) VALUES (?1)", params![file])?;

    Ok(())
}

pub fn sync_films_with_files() -> Result<(), AppError> {
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
                    "INSERT INTO films (file, directory) VALUES (?1, ?2)",
                    params![file, &directory.id],
                )?;
            }
        }
    }

    Ok(())
}
