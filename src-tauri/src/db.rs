use std::collections::HashMap;
use std::path::Path;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::error::AppResult;
use crate::scanner;

pub type Db = SqlitePool;

pub async fn open(app_data_dir: &Path) -> AppResult<Db> {
    std::fs::create_dir_all(app_data_dir)?;
    let path = app_data_dir.join("rustflix.sqlite");

    let opts = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    post_migration_fixups(&pool).await?;

    Ok(pool)
}

/// Runs once after every migration pass. Handles dedup of pre-0002 shows
/// (the legacy fingerprint backfill + duplicate merge), ensures the
/// unique index exists, and renames any legacy `auth_required` sentinel
/// in `metadata_jobs` to `tmdb_auth_required`. Idempotent — re-runs
/// every startup but only writes when rows actually need fixing.
async fn post_migration_fixups(pool: &SqlitePool) -> AppResult<()> {
    let stale: Vec<(i64, i64, String)> =
        sqlx::query_as("SELECT id, library_id, title FROM shows WHERE fingerprint = ''")
            .fetch_all(pool)
            .await?;

    if !stale.is_empty() {
        let mut groups: HashMap<(i64, String), Vec<(i64, String)>> = HashMap::new();

        for (id, library_id, title) in stale {
            let stripped = scanner::strip_season_suffix(&title);
            let key = scanner::fingerprint(&title);
            groups.entry((library_id, key)).or_default().push((id, stripped));
        }

        for ((_library_id, fingerprint_value), members) in groups {
            let mut ranked: Vec<(i64, String, i64)> = Vec::with_capacity(members.len());
            for (id, stripped) in members {
                let episode_count: i64 =
                    sqlx::query_scalar("SELECT COUNT(*) FROM episodes WHERE show_id = ?1")
                        .bind(id)
                        .fetch_one(pool)
                        .await?;
                ranked.push((id, stripped, episode_count));
            }
            ranked.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));

            let (canonical_id, canonical_title, _) = ranked.first().cloned().unwrap();

            for (other_id, _, _) in ranked.iter().skip(1) {
                sqlx::query(
                    "DELETE FROM episodes WHERE show_id = ?1 AND EXISTS (
                        SELECT 1 FROM episodes existing
                        WHERE existing.show_id = ?2
                          AND existing.season = episodes.season
                          AND existing.episode = episodes.episode
                     )",
                )
                .bind(other_id)
                .bind(canonical_id)
                .execute(pool)
                .await?;

                sqlx::query("UPDATE episodes SET show_id = ?1 WHERE show_id = ?2")
                    .bind(canonical_id)
                    .bind(other_id)
                    .execute(pool)
                    .await?;

                sqlx::query("DELETE FROM shows WHERE id = ?1")
                    .bind(other_id)
                    .execute(pool)
                    .await?;
            }

            sqlx::query("UPDATE shows SET title = ?1, fingerprint = ?2 WHERE id = ?3")
                .bind(&canonical_title)
                .bind(&fingerprint_value)
                .bind(canonical_id)
                .execute(pool)
                .await?;
        }
    }

    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_shows_library_fingerprint
         ON shows(library_id, fingerprint)",
    )
    .execute(pool)
    .await?;

    // One-shot: rename the legacy auth_required sentinel that PRs
    // before the IMDB-fallback work emitted. Idempotent.
    sqlx::query(
        "UPDATE metadata_jobs SET last_error = 'tmdb_auth_required'
         WHERE last_error = 'auth_required'",
    )
    .execute(pool)
    .await?;

    Ok(())
}
