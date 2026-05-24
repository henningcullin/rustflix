-- Adds the metadata sync columns on shows and movies plus the job-queue and
-- app-settings tables. Pure ALTER TABLE ADD COLUMN + CREATE TABLE / INDEX,
-- so no table rebuild is needed.

ALTER TABLE shows  ADD COLUMN provider           TEXT;
ALTER TABLE shows  ADD COLUMN provider_id        TEXT;
ALTER TABLE shows  ADD COLUMN rating             REAL;
ALTER TABLE shows  ADD COLUMN genres             TEXT;
ALTER TABLE shows  ADD COLUMN top_cast           TEXT;
ALTER TABLE shows  ADD COLUMN first_air_date     TEXT;
ALTER TABLE shows  ADD COLUMN metadata_synced_at INTEGER;
ALTER TABLE shows  ADD COLUMN metadata_locked    INTEGER NOT NULL DEFAULT 0;

ALTER TABLE movies ADD COLUMN provider           TEXT;
ALTER TABLE movies ADD COLUMN provider_id        TEXT;
ALTER TABLE movies ADD COLUMN rating             REAL;
ALTER TABLE movies ADD COLUMN genres             TEXT;
ALTER TABLE movies ADD COLUMN top_cast           TEXT;
ALTER TABLE movies ADD COLUMN runtime_minutes    INTEGER;
ALTER TABLE movies ADD COLUMN metadata_synced_at INTEGER;
ALTER TABLE movies ADD COLUMN metadata_locked    INTEGER NOT NULL DEFAULT 0;

CREATE UNIQUE INDEX idx_shows_provider
    ON shows(provider, provider_id)  WHERE provider IS NOT NULL;
CREATE UNIQUE INDEX idx_movies_provider
    ON movies(provider, provider_id) WHERE provider IS NOT NULL;

CREATE TABLE metadata_jobs (
    kind            TEXT    NOT NULL,
    media_id        INTEGER NOT NULL,
    enqueued_at     INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    attempts        INTEGER NOT NULL DEFAULT 0,
    last_error      TEXT,
    next_attempt_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    PRIMARY KEY (kind, media_id)
);

CREATE TABLE IF NOT EXISTS app_settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
