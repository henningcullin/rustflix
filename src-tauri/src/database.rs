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
            file  TEXT NOT NULL,
            link  TEXT,
            title TEXT,
            release_year INTEGER,
            duration INTEGER,
            cover_image TEXT
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
