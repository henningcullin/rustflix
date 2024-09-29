use std::fs;

use rusqlite::{params, OptionalExtension};

use crate::{
    database::create_connection,
    directories::{actions::get_all_directories, Directory},
    error::AppError,
    FromRow,
};

use super::Film;

pub fn get_all_films() -> Result<Vec<Film>, AppError> {
    let conn = create_connection()?;

    let mut stmt = conn.prepare(r#"--sql
        SELECT 
            f.id as film_id, f.file, f.imdb_id, f.title, 
            f.release_date, f.plot, f.run_time, f.has_color, f.rating, 
            f.has_watched, f.left_off_point, f.registered,
            (d.id || ':' || d.path) as directory,
            GROUP_CONCAT(DISTINCT g.id || ':' || g.name) as genres,
            GROUP_CONCAT(DISTINCT l.id || ':' || l.name) as languages,
            GROUP_CONCAT(DISTINCT k.keyword) as keywords,
            GROUP_CONCAT(DISTINCT 
                COALESCE(p.id, '') || ':' || COALESCE(p.imdb_id, '') || ':' || COALESCE(p.name, '') || ':' || 
                COALESCE(p.age, '') || ':' || COALESCE(p.gender, '') || ':' || COALESCE(p.birthplace, '')
            ) as directors,
            GROUP_CONCAT(DISTINCT 
                COALESCE(c.film_id, '') || ':' || COALESCE(c.description, '') || ':' || COALESCE(ap.id, '') || ':' || 
                COALESCE(ap.imdb_id, '') || ':' || COALESCE(ap.name, '') || ':' || COALESCE(ap.age, '') || ':' || 
                COALESCE(ap.gender, '') || ':' || COALESCE(ap.birthplace, '')
            ) as stars
        FROM films f
        LEFT JOIN directories d ON f.directory = d.id
        LEFT JOIN film_genres fg ON f.id = fg.film_id
        LEFT JOIN genres g ON fg.genre_id = g.id
        LEFT JOIN film_languages fl ON f.id = fl.film_id
        LEFT JOIN languages l ON fl.language_id = l.id
        LEFT JOIN film_keywords k ON f.id = k.film_id
        LEFT JOIN film_directors fd ON f.id = fd.film_id
        LEFT JOIN persons p ON fd.person_id = p.id
        LEFT JOIN characters c ON f.id = c.film_id
        LEFT JOIN persons ap ON c.actor = ap.id
            GROUP BY f.id
    "#)?;

    let films = stmt
        .query_map([], Film::from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(films)
}

pub fn get_film(id: u32) -> Result<Film, AppError> {
    let conn = create_connection()?;

    let mut stmt = conn.prepare("--sql
        SELECT 
            f.id as film_id, f.file, f.imdb_id, f.title, 
            f.release_date, f.plot, f.run_time, f.has_color, f.rating, 
            f.has_watched, f.left_off_point, f.registered,
            (d.id || ':' || d.path) as directory,
            GROUP_CONCAT(DISTINCT g.id || ':' || g.name) as genres,
            GROUP_CONCAT(DISTINCT l.id || ':' || l.name) as languages,
            GROUP_CONCAT(DISTINCT k.keyword) as keywords,
            GROUP_CONCAT(DISTINCT 
                COALESCE(p.id, '') || ':' || COALESCE(p.imdb_id, '') || ':' || COALESCE(p.name, '') || ':' || 
                COALESCE(p.age, '') || ':' || COALESCE(p.gender, '') || ':' || COALESCE(p.birthplace, '')
            ) as directors,
            GROUP_CONCAT(DISTINCT 
                COALESCE(c.film_id, '') || ':' || COALESCE(c.description, '') || ':' || COALESCE(ap.id, '') || ':' || 
                COALESCE(ap.imdb_id, '') || ':' || COALESCE(ap.name, '') || ':' || COALESCE(ap.age, '') || ':' || 
                COALESCE(ap.gender, '') || ':' || COALESCE(ap.birthplace, '')
            ) as stars
        FROM films f
        LEFT JOIN directories d ON f.directory = d.id
        LEFT JOIN film_genres fg ON f.id = fg.film_id
        LEFT JOIN genres g ON fg.genre_id = g.id
        LEFT JOIN film_languages fl ON f.id = fl.film_id
        LEFT JOIN languages l ON fl.language_id = l.id
        LEFT JOIN film_keywords k ON f.id = k.film_id
        LEFT JOIN film_directors fd ON f.id = fd.film_id
        LEFT JOIN persons p ON fd.person_id = p.id
        LEFT JOIN characters c ON f.id = c.film_id
        LEFT JOIN persons ap ON c.actor = ap.id
            WHERE f.id = ?1")?;
    let film = stmt.query_row([id], Film::from_row)?;

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

pub fn sync_new_films() -> Result<(), AppError> {
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

    /* print_all_tables(&conn)?; */

    Ok(())
}

// DEBUG FUNCTION
pub fn _print_all_tables(conn: &rusqlite::Connection) -> Result<(), AppError> {
    // Define the list of tables and their respective queries
    let tables = vec![
        ("directories", "SELECT * FROM directories"),
        ("genres", "SELECT * FROM genres"),
        ("languages", "SELECT * FROM languages"),
        ("persons", "SELECT * FROM persons"),
        ("characters", "SELECT * FROM characters"),
        ("films", "SELECT * FROM films"),
        ("film_genres", "SELECT * FROM film_genres"),
        ("film_languages", "SELECT * FROM film_languages"),
        ("film_keywords", "SELECT * FROM film_keywords"),
        ("film_directors", "SELECT * FROM film_directors"),
    ];

    for (table_name, query) in tables {
        println!("--- Contents of table: {} ---", table_name);

        let mut stmt = conn.prepare(query)?;

        // Get the column names before iterating over the rows
        let column_names: Vec<String> = (0..stmt.column_count())
            .map(|i| stmt.column_name(i).unwrap_or("").to_string())
            .collect();

        let rows = stmt.query_map([], |row| {
            let mut values = Vec::new();
            for i in 0..row.as_ref().column_count() {
                values.push(row.get::<usize, rusqlite::types::Value>(i)?);
            }
            Ok(values)
        })?;

        for row in rows {
            match row {
                Ok(values) => {
                    for (i, value) in values.iter().enumerate() {
                        print!("{}: {:?} ", column_names[i], value);
                    }
                    println!();
                }
                Err(e) => println!("Failed to retrieve row: {:?}", e),
            }
        }
        println!(); // Add an empty line for better readability
    }

    let test_row_1 = conn.query_row(
        r#"
    SELECT persons.name
        FROM persons
    JOIN characters ON persons.id = characters.actor
        WHERE characters.film_id = 5;
    "#,
        [],
        |row| row.get::<_, String>(0),
    )?;

    println!("{:?}", test_row_1);

    let test_row_2 = conn.query_row(
        r#"
    SELECT p.name, c.description
    FROM persons p
    JOIN characters c ON p.id = c.actor
    JOIN films f ON c.film_id = f.id
    WHERE f.title IS NOT NULL;
    "#,
        [],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    )?;

    println!("{:?}", test_row_2);

    Ok(())
}
