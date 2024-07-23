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

    // Enable foreign key support
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Create directories table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS directories (
            id   INTEGER PRIMARY KEY,
            path TEXT NOT NULL
        )",
        [],
    )?;

    // Create films table with a foreign key to directories table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS films (
            id          INTEGER PRIMARY KEY,
            file        TEXT NOT NULL,
            directory   INTEGER NOT NULL,
            link        TEXT,
            title       TEXT,
            synopsis    TEXT,
            release_date TEXT,
            duration    INTEGER,
            cover_image TEXT,
            registered  INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (directory) REFERENCES directories(id) ON DELETE CASCADE
        )",
        [],
    )?;

    Ok(())
}
