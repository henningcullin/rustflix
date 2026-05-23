CREATE TABLE IF NOT EXISTS libraries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    kind TEXT NOT NULL DEFAULT 'mixed',
    added_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE IF NOT EXISTS shows (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    year INTEGER,
    folder_path TEXT NOT NULL UNIQUE,
    poster_path TEXT,
    overview TEXT,
    added_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE IF NOT EXISTS movies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    year INTEGER,
    path TEXT NOT NULL UNIQUE,
    poster_path TEXT,
    overview TEXT,
    duration_seconds INTEGER,
    added_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE IF NOT EXISTS episodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    show_id INTEGER NOT NULL REFERENCES shows(id) ON DELETE CASCADE,
    season INTEGER NOT NULL,
    episode INTEGER NOT NULL,
    title TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    duration_seconds INTEGER,
    added_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    UNIQUE(show_id, season, episode)
);

CREATE TABLE IF NOT EXISTS watch_history (
    media_kind TEXT NOT NULL,
    media_id INTEGER NOT NULL,
    progress_seconds INTEGER NOT NULL DEFAULT 0,
    duration_seconds INTEGER,
    watched INTEGER NOT NULL DEFAULT 0,
    last_watched_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    PRIMARY KEY (media_kind, media_id)
);

CREATE INDEX IF NOT EXISTS idx_episodes_show ON episodes(show_id, season, episode);
CREATE INDEX IF NOT EXISTS idx_watch_recent ON watch_history(last_watched_at DESC);
