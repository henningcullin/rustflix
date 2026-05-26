use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::{
    ContinueWatchingItem, Episode, EpisodeRef, Library, LibraryKind, MergeOutcome, Movie, Season,
    Show,
};

pub async fn list_libraries(pool: &SqlitePool) -> AppResult<Vec<Library>> {
    let rows = sqlx::query_as::<_, Library>("SELECT id, path, kind FROM libraries ORDER BY id")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn add_library(pool: &SqlitePool, path: &str, kind: LibraryKind) -> AppResult<Library> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO libraries (path, kind) VALUES (?1, ?2)
         ON CONFLICT(path) DO UPDATE SET kind = excluded.kind
         RETURNING id",
    )
    .bind(path)
    .bind(kind)
    .fetch_one(pool)
    .await?;

    Ok(Library {
        id,
        path: path.to_string(),
        kind,
    })
}

pub async fn remove_library(pool: &SqlitePool, id: i64) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM libraries WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::LibraryNotFound(id));
    }
    Ok(())
}

const MOVIE_SELECT: &str = "
    SELECT m.id, m.title, m.year, m.path, m.poster_path, m.poster_origin, m.overview,
           m.duration_seconds,
           COALESCE(w.progress_seconds, 0) AS progress_seconds,
           COALESCE(w.watched, 0) AS watched,
           m.added_at,
           m.provider, m.provider_id, m.rating, m.genres, m.top_cast,
           m.runtime_minutes, m.metadata_synced_at, m.metadata_locked
    FROM movies m
    LEFT JOIN watch_history w
      ON w.media_kind = 'movie' AND w.media_id = m.id
";

pub async fn list_movies(pool: &SqlitePool) -> AppResult<Vec<Movie>> {
    let sql = format!("{MOVIE_SELECT} ORDER BY m.title COLLATE NOCASE");
    let rows = sqlx::query_as::<_, Movie>(&sql).fetch_all(pool).await?;
    Ok(rows)
}

pub async fn get_movie(pool: &SqlitePool, id: i64) -> AppResult<Movie> {
    let sql = format!("{MOVIE_SELECT} WHERE m.id = ?1");
    let row = sqlx::query_as::<_, Movie>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?;
    row.ok_or(AppError::MediaNotFound(id))
}

const SHOW_SELECT: &str = "
    SELECT s.id, s.library_id, s.title, s.year, s.folder_path, s.fingerprint,
           s.poster_path, s.poster_origin, s.overview,
           (SELECT COUNT(*) FROM episodes e WHERE e.show_id = s.id) AS episode_count,
           (SELECT COUNT(*) FROM episodes e
              LEFT JOIN watch_history w
                ON w.media_kind = 'episode' AND w.media_id = e.id
             WHERE e.show_id = s.id AND COALESCE(w.watched, 0) = 1) AS watched_count,
           s.added_at,
           s.provider, s.provider_id, s.rating, s.genres, s.top_cast,
           s.first_air_date, s.metadata_synced_at, s.metadata_locked
    FROM shows s
";

pub async fn list_shows(pool: &SqlitePool) -> AppResult<Vec<Show>> {
    let sql = format!("{SHOW_SELECT} ORDER BY s.title COLLATE NOCASE");
    let rows = sqlx::query_as::<_, Show>(&sql).fetch_all(pool).await?;
    Ok(rows)
}

pub async fn get_show(pool: &SqlitePool, id: i64) -> AppResult<Show> {
    let sql = format!("{SHOW_SELECT} WHERE s.id = ?1");
    let row = sqlx::query_as::<_, Show>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?;
    row.ok_or(AppError::MediaNotFound(id))
}

const EPISODE_SELECT: &str = "
    SELECT e.id, e.show_id, e.season, e.episode, e.title, e.path,
           e.duration_seconds,
           COALESCE(w.progress_seconds, 0) AS progress_seconds,
           COALESCE(w.watched, 0) AS watched
    FROM episodes e
    LEFT JOIN watch_history w
      ON w.media_kind = 'episode' AND w.media_id = e.id
";

pub async fn list_seasons(pool: &SqlitePool, show_id: i64) -> AppResult<Vec<Season>> {
    let sql = format!("{EPISODE_SELECT} WHERE e.show_id = ?1 ORDER BY e.season, e.episode");
    let episodes = sqlx::query_as::<_, Episode>(&sql)
        .bind(show_id)
        .fetch_all(pool)
        .await?;

    let mut grouped: Vec<Season> = Vec::new();
    for ep in episodes {
        if let Some(last) = grouped.last_mut() {
            if last.season == ep.season {
                last.episodes.push(ep);
                continue;
            }
        }
        grouped.push(Season {
            season: ep.season,
            episodes: vec![ep],
        });
    }
    Ok(grouped)
}

pub async fn get_episode(pool: &SqlitePool, id: i64) -> AppResult<Episode> {
    let sql = format!("{EPISODE_SELECT} WHERE e.id = ?1");
    let row = sqlx::query_as::<_, Episode>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?;
    row.ok_or(AppError::MediaNotFound(id))
}

pub async fn upsert_progress(
    pool: &SqlitePool,
    kind: &str,
    id: i64,
    progress: i64,
    duration: Option<i64>,
    watched: bool,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO watch_history (media_kind, media_id, progress_seconds, duration_seconds, watched, last_watched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, strftime('%s','now'))
         ON CONFLICT(media_kind, media_id) DO UPDATE SET
            progress_seconds = excluded.progress_seconds,
            duration_seconds = COALESCE(excluded.duration_seconds, watch_history.duration_seconds),
            watched = excluded.watched,
            last_watched_at = excluded.last_watched_at",
    )
    .bind(kind)
    .bind(id)
    .bind(progress)
    .bind(duration)
    .bind(watched as i64)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_show_metadata(
    pool: &SqlitePool,
    id: i64,
    title: Option<&str>,
    year: Option<i32>,
    overview: Option<&str>,
) -> AppResult<()> {
    update_metadata_row(pool, "shows", id, title, year, overview).await
}

pub async fn update_movie_metadata(
    pool: &SqlitePool,
    id: i64,
    title: Option<&str>,
    year: Option<i32>,
    overview: Option<&str>,
) -> AppResult<()> {
    update_metadata_row(pool, "movies", id, title, year, overview).await
}

/// Shared body for `update_show_metadata` and `update_movie_metadata`. Only
/// fields passed as `Some` are touched. There's no v1 way to *clear* a field
/// to NULL through this API; that needs an explicit "reset" command if it
/// turns out users want it.
async fn update_metadata_row(
    pool: &SqlitePool,
    table: &str,
    id: i64,
    title: Option<&str>,
    year: Option<i32>,
    overview: Option<&str>,
) -> AppResult<()> {
    let mut assignments: Vec<&str> = Vec::new();
    if title.is_some() {
        assignments.push("title = ?");
    }
    if year.is_some() {
        assignments.push("year = ?");
    }
    if overview.is_some() {
        assignments.push("overview = ?");
    }
    if assignments.is_empty() {
        return Ok(());
    }

    let sql = format!(
        "UPDATE {table} SET {}, metadata_locked = 1 WHERE id = ?",
        assignments.join(", ")
    );

    let mut query = sqlx::query(&sql);
    if let Some(value) = title {
        query = query.bind(value);
    }
    if let Some(value) = year {
        query = query.bind(value);
    }
    if let Some(value) = overview {
        query = query.bind(value);
    }
    let result = query.bind(id).execute(pool).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::MediaNotFound(id));
    }
    Ok(())
}

pub async fn set_show_poster(
    pool: &SqlitePool,
    id: i64,
    path: &str,
    origin: &str,
) -> AppResult<()> {
    set_poster_row(pool, "shows", id, path, origin).await
}

pub async fn set_movie_poster(
    pool: &SqlitePool,
    id: i64,
    path: &str,
    origin: &str,
) -> AppResult<()> {
    set_poster_row(pool, "movies", id, path, origin).await
}

pub async fn reset_show_poster(pool: &SqlitePool, id: i64) -> AppResult<()> {
    reset_poster_row(pool, "shows", id).await
}

pub async fn reset_movie_poster(pool: &SqlitePool, id: i64) -> AppResult<()> {
    reset_poster_row(pool, "movies", id).await
}

async fn set_poster_row(
    pool: &SqlitePool,
    table: &str,
    id: i64,
    path: &str,
    origin: &str,
) -> AppResult<()> {
    let sql = format!("UPDATE {table} SET poster_path = ?1, poster_origin = ?2 WHERE id = ?3");
    let result = sqlx::query(&sql)
        .bind(path)
        .bind(origin)
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::MediaNotFound(id));
    }
    Ok(())
}

async fn reset_poster_row(pool: &SqlitePool, table: &str, id: i64) -> AppResult<()> {
    let sql = format!("UPDATE {table} SET poster_path = NULL, poster_origin = NULL WHERE id = ?1");
    let result = sqlx::query(&sql).bind(id).execute(pool).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::MediaNotFound(id));
    }
    Ok(())
}

pub async fn update_episode_title(pool: &SqlitePool, id: i64, title: &str) -> AppResult<()> {
    let result = sqlx::query("UPDATE episodes SET title = ?1 WHERE id = ?2")
        .bind(title)
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::MediaNotFound(id));
    }

    Ok(())
}

/// Reassigns every episode of `source_id` to `target_id` and deletes
/// `source_id`. If both shows have an episode with the same
/// `(season, episode)` the merge is rejected — the conflicting pairs are
/// returned in `MergeOutcome.conflicts` and nothing is changed. Caller can
/// surface the conflicts to the user and try again after they're resolved.
pub async fn merge_shows(
    pool: &SqlitePool,
    target_id: i64,
    source_id: i64,
) -> AppResult<MergeOutcome> {
    if target_id == source_id {
        return Err(AppError::Other(
            "cannot merge a show into itself".to_string(),
        ));
    }

    let conflicts: Vec<EpisodeRef> = sqlx::query_as(
        "SELECT source.season, source.episode
         FROM episodes source
         WHERE source.show_id = ?1
           AND EXISTS (
               SELECT 1 FROM episodes target
               WHERE target.show_id = ?2
                 AND target.season = source.season
                 AND target.episode = source.episode
           )
         ORDER BY source.season, source.episode",
    )
    .bind(source_id)
    .bind(target_id)
    .fetch_all(pool)
    .await?;

    if !conflicts.is_empty() {
        return Ok(MergeOutcome { conflicts });
    }

    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE episodes SET show_id = ?1 WHERE show_id = ?2")
        .bind(target_id)
        .bind(source_id)
        .execute(&mut *tx)
        .await?;
    let deleted = sqlx::query("DELETE FROM shows WHERE id = ?1")
        .bind(source_id)
        .execute(&mut *tx)
        .await?;
    if deleted.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(AppError::MediaNotFound(source_id));
    }
    tx.commit().await?;

    Ok(MergeOutcome { conflicts: vec![] })
}

/// Removes a show and all of its episodes from the library. Files on disk
/// are not touched — only DB rows are deleted. Orphan `watch_history` rows
/// for the show's episodes are cleaned up in the same transaction.
pub async fn delete_show(pool: &SqlitePool, show_id: i64) -> AppResult<()> {
    let mut tx = pool.begin().await?;

    let episode_ids: Vec<i64> = sqlx::query_scalar("SELECT id FROM episodes WHERE show_id = ?1")
        .bind(show_id)
        .fetch_all(&mut *tx)
        .await?;

    for episode_id in episode_ids {
        sqlx::query("DELETE FROM watch_history WHERE media_kind = 'episode' AND media_id = ?1")
            .bind(episode_id)
            .execute(&mut *tx)
            .await?;
    }

    let deleted = sqlx::query("DELETE FROM shows WHERE id = ?1")
        .bind(show_id)
        .execute(&mut *tx)
        .await?;
    if deleted.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(AppError::MediaNotFound(show_id));
    }

    tx.commit().await?;

    Ok(())
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for EpisodeRef {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(EpisodeRef {
            season: row.try_get("season")?,
            episode: row.try_get("episode")?,
        })
    }
}

pub async fn continue_watching(pool: &SqlitePool, limit: i64) -> AppResult<Vec<ContinueWatchingItem>> {
    // Pull recent in-progress entries for both kinds, merge by last_watched_at, then hydrate.
    let movie_rows: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT media_id, last_watched_at FROM watch_history
         WHERE media_kind = 'movie' AND watched = 0 AND progress_seconds > 30
         ORDER BY last_watched_at DESC LIMIT ?1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let episode_rows: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT media_id, last_watched_at FROM watch_history
         WHERE media_kind = 'episode' AND watched = 0 AND progress_seconds > 30
         ORDER BY last_watched_at DESC LIMIT ?1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut combined: Vec<(i64, char, i64)> = movie_rows
        .into_iter()
        .map(|(id, ts)| (ts, 'm', id))
        .chain(episode_rows.into_iter().map(|(id, ts)| (ts, 'e', id)))
        .collect();
    combined.sort_by(|a, b| b.0.cmp(&a.0));
    combined.truncate(limit as usize);

    let mut out = Vec::new();
    for (_, kind, id) in combined {
        if kind == 'm' {
            if let Ok(m) = get_movie(pool, id).await {
                out.push(ContinueWatchingItem::Movie { movie: m });
            }
        } else if let Ok(ep) = get_episode(pool, id).await {
            if let Ok(show) = get_show(pool, ep.show_id).await {
                out.push(ContinueWatchingItem::Episode { show, episode: ep });
            }
        }
    }
    Ok(out)
}

#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct NeedsReviewItem {
    pub kind: String,
    pub id: i64,
    pub title: String,
    pub year: Option<i32>,
}

pub async fn list_needs_review(pool: &SqlitePool) -> AppResult<Vec<NeedsReviewItem>> {
    let items: Vec<NeedsReviewItem> = sqlx::query_as(
        "SELECT 'show' AS kind, id, title, year FROM shows
            WHERE provider IS NULL
              AND NOT EXISTS (SELECT 1 FROM metadata_jobs j
                              WHERE j.kind = 'show' AND j.media_id = shows.id)
         UNION ALL
         SELECT 'movie' AS kind, id, title, year FROM movies
            WHERE provider IS NULL
              AND NOT EXISTS (SELECT 1 FROM metadata_jobs j
                              WHERE j.kind = 'movie' AND j.media_id = movies.id)
         ORDER BY title COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?;

    Ok(items)
}

pub async fn get_app_setting(pool: &SqlitePool, key: &str) -> AppResult<Option<String>> {
    let value: Option<String> =
        sqlx::query_scalar("SELECT value FROM app_settings WHERE key = ?1")
            .bind(key)
            .fetch_optional(pool)
            .await?;

    Ok(value)
}

pub async fn set_app_setting(pool: &SqlitePool, key: &str, value: &str) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_app_setting(pool: &SqlitePool, key: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM app_settings WHERE key = ?1")
        .bind(key)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn metadata_status_counts(
    pool: &SqlitePool,
) -> AppResult<crate::models::MetadataStatusCounts> {
    let pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM metadata_jobs
         WHERE attempts = 0
           AND COALESCE(last_error, '') NOT IN ('tmdb_auth_required', 'no_provider_available')",
    )
    .fetch_one(pool)
    .await?;

    let failed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM metadata_jobs
         WHERE attempts > 0 AND attempts < 8
           AND COALESCE(last_error, '') NOT IN ('tmdb_auth_required', 'no_provider_available')",
    )
    .fetch_one(pool)
    .await?;

    let tmdb_auth_required: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM metadata_jobs WHERE last_error = 'tmdb_auth_required'",
    )
    .fetch_one(pool)
    .await?;

    let no_provider_available: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM metadata_jobs WHERE last_error = 'no_provider_available'",
    )
    .fetch_one(pool)
    .await?;

    let dead_letter: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM metadata_jobs WHERE attempts >= 8",
    )
    .fetch_one(pool)
    .await?;

    let needs_review: i64 = sqlx::query_scalar(
        "SELECT
             (SELECT COUNT(*) FROM shows
                WHERE provider IS NULL
                  AND NOT EXISTS (SELECT 1 FROM metadata_jobs j
                                  WHERE j.kind = 'show' AND j.media_id = shows.id))
           + (SELECT COUNT(*) FROM movies
                WHERE provider IS NULL
                  AND NOT EXISTS (SELECT 1 FROM metadata_jobs j
                                  WHERE j.kind = 'movie' AND j.media_id = movies.id))",
    )
    .fetch_one(pool)
    .await?;

    Ok(crate::models::MetadataStatusCounts {
        pending,
        failed,
        tmdb_auth_required,
        no_provider_available,
        dead_letter,
        needs_review,
    })
}
