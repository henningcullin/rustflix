use std::{fs, path::PathBuf};

use dirs::data_local_dir;
use rusqlite::{params, Connection};
use serde::Serialize;

#[derive(Serialize)]
pub struct Film {
    pub id: u32,
    pub title: Option<String>,
    pub file: String,
    pub link: Option<String>,
}

fn get_database_path() -> PathBuf {
    let mut path = data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("Rustflix");
    if !path.exists() {
        fs::create_dir_all(&path).expect("Failed to create appdata directory");
    }

    path.push("database.sqlite");
    path
}

fn create_connection() -> Result<Connection, rusqlite::Error> {
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

pub fn add_directory(path: &str) -> Result<(), rusqlite::Error> {
    let conn = create_connection()?;

    conn.execute("INSERT INTO directories (path) VALUES (?1)", params![path])?;

    Ok(())
}
