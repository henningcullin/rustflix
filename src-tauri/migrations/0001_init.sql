PRAGMA foreign_keys = ON;

CREATE TABLE directories (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    path        TEXT NOT NULL UNIQUE,
    recursive   INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE films (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path       TEXT NOT NULL UNIQUE,
    tmdb_id         INTEGER UNIQUE,
    imdb_id         TEXT,
    title           TEXT NOT NULL,
    original_title  TEXT,
    overview        TEXT,
    release_date    TEXT,
    runtime         INTEGER,
    rating          REAL,
    poster_path     TEXT,
    backdrop_path   TEXT,
    left_off_point  INTEGER NOT NULL DEFAULT 0,
    watched         INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE genres (
    id      INTEGER PRIMARY KEY,
    name    TEXT NOT NULL UNIQUE
);

CREATE TABLE film_genres (
    film_id     INTEGER NOT NULL,
    genre_id    INTEGER NOT NULL,
    PRIMARY KEY (film_id, genre_id),
    FOREIGN KEY (film_id)  REFERENCES films(id)  ON DELETE CASCADE,
    FOREIGN KEY (genre_id) REFERENCES genres(id) ON DELETE CASCADE
);

CREATE TABLE persons (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tmdb_id         INTEGER UNIQUE,
    name            TEXT NOT NULL,
    profile_path    TEXT,
    created_at      TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE film_persons (
    film_id     INTEGER NOT NULL,
    person_id   INTEGER NOT NULL,
    role        TEXT NOT NULL,
    character   TEXT,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (film_id, person_id, role),
    FOREIGN KEY (film_id)   REFERENCES films(id)   ON DELETE CASCADE,
    FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE
);

CREATE INDEX idx_film_persons_film   ON film_persons(film_id);
CREATE INDEX idx_film_persons_person ON film_persons(person_id);
CREATE INDEX idx_film_genres_film    ON film_genres(film_id);
