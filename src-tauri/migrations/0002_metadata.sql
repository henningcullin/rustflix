-- no-transaction
-- Rebuilds `shows` to drop `folder_path UNIQUE` (sqlite can't drop a UNIQUE
-- constraint in place) and to add `fingerprint` (stable dedup key) and
-- `poster_origin` (tracks whether a poster was discovered by the scanner
-- or uploaded by the user). Also adds `poster_origin` to `movies`.
--
-- A UNIQUE INDEX on (library_id, fingerprint) is created later, by
-- `db::dedupe_shows_and_index`, after that function backfills fingerprints
-- and merges any pre-existing duplicate shows. The migration itself only
-- changes schema.

PRAGMA foreign_keys = OFF;

CREATE TABLE shows_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    year INTEGER,
    folder_path TEXT NOT NULL,
    poster_path TEXT,
    overview TEXT,
    fingerprint TEXT NOT NULL DEFAULT '',
    poster_origin TEXT,
    added_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

INSERT INTO shows_new (id, library_id, title, year, folder_path, poster_path, overview, added_at)
SELECT id, library_id, title, year, folder_path, poster_path, overview, added_at
FROM shows;

DROP TABLE shows;

ALTER TABLE shows_new RENAME TO shows;

ALTER TABLE movies ADD COLUMN poster_origin TEXT;

PRAGMA foreign_keys = ON;
