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

    conn.execute_batch(
        r#"--sql
        PRAGMA foreign_keys = ON

        CREATE TABLE IF NOT EXISTS directories (
            id   INTEGER PRIMARY KEY,
            path TEXT NOT NULL
        )

        CREATE TABLE IF NOT EXISTS genres (
            id  INTEGER PRIMARY KEY,
            path TEXT NOT NULL
        )

        CREATE TABLE IF NOT EXISTS languages (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        )

        CREATE TABLE IF NOT EXISTS persons (
            id INTEGER PRIMARY KEY,
            imdb_id TEXT,
            avatar TEXT,
            age INTEGER,
            gender INTEGER,
            birthplace INTEGER
        )

        CREATE TABLE IF NOT EXISTS characters (
            id INTEGER PRIMARY KEY,
            actor INTEGER NOT NULL,
            description TEXT NOT NULL,
            FOREIGN KEY (actor) REFERENCES persons(id) ON DELETE 
        )

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
            has_watched     INTEGER,
            left_off_point  INTEGER,
            registered  INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (directory) REFERENCES directories(id) ON DELETE CASCADE
        )
        
        CREATE TABLE IF NOT EXISTS film_genres (
            film_id INTEGER NOT NULL,
            genre_id INTEGER NOT NULL,
            FOREIGN KEY (film_id) REFERENCES films(id) ON DELETE CASCADE,
            FOREIGN KEY (genre_id) REFERENCES genres(id) ON DELETE CASCADE,
            PRIMARY KEY (film_id, genre_id)
        );

        CREATE TABLE IF NOT EXISTS film_languages (
            film_id INTEGER NOT NULL,
            language_id INTEGER NOT NULL,
            FOREIGN KEY (film_id) REFERENCES films(id) ON DELETE CASCADE,
            FOREIGN KEY (language_id) REFERENCES languages(id) ON DELETE CASCADE,
            PRIMARY KEY (film_id, language_id)
        );

        CREATE TABLE IF NOT EXISTS film_keywords (
            film_id INTEGER NOT NULL,
            keyword TEXT NOT NULL,
            FOREIGN KEY (film_id) REFERENCES films(id) ON DELETE CASCADE,
            PRIMARY KEY (film_id, keyword)
        );

        CREATE TABLE IF NOT EXISTS film_directors (
            film_id INTEGER NOT NULL,
            person_id INTEGER NOT NULL,
            FOREIGN KEY (film_id) REFERENCES films(id) ON DELETE CASCADE,
            FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE,
            PRIMARY KEY (film_id, person_id)
        );

        CREATE TABLE IF NOT EXISTS film_characters (
            film_id INTEGER NOT NULL,
            character_id INTEGER NOT NULL,
            FOREIGN KEY (film_id) REFERENCES films(id) ON DELETE CASCADE,
            FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE,
            PRIMARY KEY (film_id, character_id)
        );
        "#,
    )?;

    Ok(())
}
