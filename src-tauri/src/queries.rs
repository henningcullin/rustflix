use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::{
    ContinueWatchingItem, Episode, Library, LibraryKind, Movie, Season, Show,
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
    let res = sqlx::query("DELETE FROM libraries WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::LibraryNotFound(id));
    }
    Ok(())
}

const MOVIE_SELECT: &str = "
    SELECT m.id, m.title, m.year, m.path, m.poster_path, m.overview,
           m.duration_seconds,
           COALESCE(w.progress_seconds, 0) AS progress_seconds,
           COALESCE(w.watched, 0) AS watched,
           m.added_at
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
    SELECT s.id, s.title, s.year, s.folder_path, s.poster_path, s.overview,
           (SELECT COUNT(*) FROM episodes e WHERE e.show_id = s.id) AS episode_count,
           (SELECT COUNT(*) FROM episodes e
              LEFT JOIN watch_history w
                ON w.media_kind = 'episode' AND w.media_id = e.id
             WHERE e.show_id = s.id AND COALESCE(w.watched, 0) = 1) AS watched_count,
           s.added_at
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
