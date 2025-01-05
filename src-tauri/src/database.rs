use std::{fs, path::PathBuf};

use dirs::data_local_dir;
use rusqlite::Connection;

use crate::error::AppError;

fn get_local_path() -> PathBuf {
    let mut path = data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("Rustflix");
    if !path.exists() {
        fs::create_dir_all(&path).expect("Failed to create data directory");
    }
    path
}

fn get_images_path() -> PathBuf {
    let mut path = get_local_path();
    path.push("images");
    if !path.exists() {
        fs::create_dir_all(&path).expect("Failed to create images directory");
    }
    path
}

pub fn get_cover_path() -> PathBuf {
    let mut path = get_images_path();
    path.push("covers");
    if !path.exists() {
        fs::create_dir_all(&path).expect("Failed to create covers directory");
    }
    path
}

pub fn get_avatar_path() -> PathBuf {
    let mut path = get_images_path();
    path.push("avatars");
    if !path.exists() {
        fs::create_dir_all(&path).expect("Failed to create avatars directory");
    }
    path
}

fn get_database_path() -> PathBuf {
    let mut path = get_local_path();
    path.push("database.sqlite");
    path
}

pub fn create_connection() -> Result<Connection, rusqlite::Error> {
    let db_path = get_database_path();
    Connection::open(db_path)
}

pub fn initialize_database() -> Result<(), rusqlite::Error> {
    let conn = create_connection()?;

    conn.execute(
        r#"--sql
        PRAGMA foreign_keys = ON
        "#,
        [],
    )?;

    conn.execute(
        r#"--sql
        CREATE TABLE IF NOT EXISTS directories (
            id INTEGER PRIMARY KEY,
            recursive INTEGER NOT NULL DEFAULT 0,
            path TEXT NOT NULL UNIQUE
        )"#,
        [],
    )?;

    conn.execute(
        r#"--sql
        CREATE TABLE IF NOT EXISTS genres (
            id  INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        )"#,
        [],
    )?;

    conn.execute(
        r#"--sql
        CREATE TABLE IF NOT EXISTS languages (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        )"#,
        [],
    )?;

    conn.execute(
        r#"--sql
        CREATE TABLE IF NOT EXISTS keywords (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        )"#,
        [],
    )?;

    conn.execute(
        r#"--sql
        CREATE TABLE IF NOT EXISTS persons (
            id INTEGER PRIMARY KEY,
            imdb_id TEXT UNIQUE,
            name TEXT,
            age INTEGER,
            gender INTEGER,
            birthplace INTEGER
        )"#,
        [],
    )?;

    conn.execute(
        r#"--sql
        CREATE TABLE IF NOT EXISTS characters (
            film_id INTEGER NOT NULL,
            description TEXT NOT NULL,
            actor INTEGER NOT NULL,
            FOREIGN KEY (film_id) REFERENCES films(id) ON DELETE CASCADE,
            FOREIGN KEY (actor) REFERENCES persons(id) ON DELETE CASCADE,
            PRIMARY KEY (film_id, actor)
        )"#,
        [],
    )?;

    conn.execute(
        r#"--sql
        CREATE TABLE IF NOT EXISTS films (
            id              INTEGER PRIMARY KEY,
            file            TEXT NOT NULL,
            imdb_id         TEXT,
            title           TEXT,
            release_date    TEXT,
            plot            TEXT,
            run_time        INTEGER,
            has_color       INTEGER,
            rating          REAL,
            has_watched     INTEGER NOT NULL DEFAULT 0,
            left_off_point  INTEGER NOT NULL DEFAULT 0,
            registered      INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (directory) REFERENCES directories(id) ON DELETE CASCADE
        )"#,
        [],
    )?;

    conn.execute(
        r#"--sql
        CREATE TABLE IF NOT EXISTS film_genres (
            film_id INTEGER NOT NULL,
            genre_id INTEGER NOT NULL,
            FOREIGN KEY (film_id) REFERENCES films(id) ON DELETE CASCADE,
            FOREIGN KEY (genre_id) REFERENCES genres(id) ON DELETE CASCADE,
            PRIMARY KEY (film_id, genre_id)
        )"#,
        [],
    )?;

    conn.execute(
        r#"--sql
        CREATE TABLE IF NOT EXISTS film_languages (
            film_id INTEGER NOT NULL,
            language_id INTEGER NOT NULL,
            FOREIGN KEY (film_id) REFERENCES films(id) ON DELETE CASCADE,
            FOREIGN KEY (language_id) REFERENCES languages(id) ON DELETE CASCADE,
            PRIMARY KEY (film_id, language_id)
        )"#,
        [],
    )?;

    conn.execute(
        r#"--sql
        CREATE TABLE IF NOT EXISTS film_keywords (
            film_id INTEGER NOT NULL,
            keyword_id INTEGER NOT NULL,
            FOREIGN KEY (film_id) REFERENCES films(id) ON DELETE CASCADE,
            FOREIGN KEY (keyword_id) REFERENCES keywords(id) ON DELETE CASCADE,
            PRIMARY KEY (film_id, keyword_id)
        )"#,
        [],
    )?;

    conn.execute(
        r#"--sql
        CREATE TABLE IF NOT EXISTS film_directors (
            film_id INTEGER NOT NULL,
            person_id INTEGER NOT NULL,
            FOREIGN KEY (film_id) REFERENCES films(id) ON DELETE CASCADE,
            FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE,
            PRIMARY KEY (film_id, person_id)
        )"#,
        [],
    )?;

    Ok(())
}

pub fn insert(
    conn: &rusqlite::Connection,
    sql: &str,
    params: &[&(dyn rusqlite::ToSql)],
) -> Result<i32, AppError> {
    let mut stmt = conn.prepare(sql)?;
    let id: i32 = stmt.query_row(params, |row| row.get(0))?;
    Ok(id)
}

pub fn delete(
    conn: &rusqlite::Connection,
    sql: &str,
    params: &[&(dyn rusqlite::ToSql)],
) -> Result<usize, AppError> {
    let mut stmt = conn.prepare(sql)?;

    let rows_affected = stmt.execute(params)?;
    Ok(rows_affected)
}

pub fn update(
    conn: &rusqlite::Connection,
    sql: &str,
    params: &[&(dyn rusqlite::ToSql)],
) -> Result<usize, AppError> {
    let mut stmt = conn.prepare(sql)?;

    let rows_affected = stmt.execute(params)?;
    Ok(rows_affected)
}
