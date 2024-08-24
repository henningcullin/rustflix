use std::{fs, path::PathBuf};

use dirs::data_local_dir;
use rusqlite::Connection;

fn get_local_path() -> PathBuf {
    let mut path = data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("Rustflix");
    if !path.exists() {
        fs::create_dir_all(&path).expect("Failed to create appdata directory");
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
            id   INTEGER PRIMARY KEY,
            path TEXT NOT NULL
        )"#,
        [],
    )?;

    conn.execute(
        r#"--sql
        CREATE TABLE IF NOT EXISTS genres (
            id  INTEGER PRIMARY KEY,
            path TEXT NOT NULL
        )"#,
        [],
    )?;

    conn.execute(
        r#"--sql
        CREATE TABLE IF NOT EXISTS languages (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        )"#,
        [],
    )?;

    conn.execute(
        r#"--sql
        CREATE TABLE IF NOT EXISTS persons (
            id INTEGER PRIMARY KEY,
            imdb_id TEXT,
            avatar TEXT,
            age INTEGER,
            gender INTEGER,
            birthplace INTEGER
        )"#,
        [],
    )?;

    conn.execute(
        r#"--sql
        CREATE TABLE IF NOT EXISTS characters (
            id INTEGER PRIMARY KEY,
            actor INTEGER NOT NULL,
            description TEXT NOT NULL,
            FOREIGN KEY (actor) REFERENCES persons(id) ON DELETE CASCADE
        )"#,
        [],
    )?;

    conn.execute(
        r#"--sql
        CREATE TABLE IF NOT EXISTS films (
            id              INTEGER PRIMARY KEY,
            file            TEXT NOT NULL,
            directory       INTEGER NOT NULL,
            imdb_id         TEXT,
            title           TEXT,
            release_date    TEXT,
            plot            TEXT,
            run_time        INTEGER,
            color           INTEGER,
            rating          INTEGER,
            cover_image     TEXT,
            has_watched     INTEGER NOT NULL DEFAULT 0,
            left_off_point  INTEGER,
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
            keyword TEXT NOT NULL,
            FOREIGN KEY (film_id) REFERENCES films(id) ON DELETE CASCADE,
            PRIMARY KEY (film_id, keyword)
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

    conn.execute(
        r#"--sql
        CREATE TABLE IF NOT EXISTS film_characters (
            film_id INTEGER NOT NULL,
            character_id INTEGER NOT NULL,
            FOREIGN KEY (film_id) REFERENCES films(id) ON DELETE CASCADE,
            FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE,
            PRIMARY KEY (film_id, character_id)
        )"#,
        [],
    )?;

    Ok(())
}
