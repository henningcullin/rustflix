//! Background worker that drains `metadata_jobs`. One task, sequential,
//! 250ms pacing between requests. Waits on a Notify when the queue is
//! empty, when there's no API key, or when every job is parked on
//! auth_required.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};
use tokio::sync::Notify;
use tokio::time::sleep;

use crate::error::{AppError, AppResult};
use crate::metadata::{apply, matching, queries, tmdb};
use crate::queries as app_queries;

const PACING_MS: u64 = 250;

pub fn spawn(pool: SqlitePool, http: reqwest::Client, app: AppHandle) -> Arc<Notify> {
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();

    // Tauri's setup closure runs outside a tokio runtime context, so a bare
    // `tokio::spawn` panics. `tauri::async_runtime::spawn` is the same
    // multi-threaded tokio runtime Tauri uses for command handlers.
    tauri::async_runtime::spawn(async move {
        if let Err(error) = run(pool, http, app, notify_clone).await {
            eprintln!("metadata worker exited with error: {error}");
        }
    });

    notify
}

async fn run(
    pool: SqlitePool,
    http: reqwest::Client,
    app: AppHandle,
    notify: Arc<Notify>,
) -> AppResult<()> {
    loop {
        let api_key = app_queries::get_app_setting(&pool, "tmdb_api_key").await?;
        let Some(api_key) = api_key else {
            notify.notified().await;
            continue;
        };

        let Some(job) = queries::next_due(&pool).await? else {
            notify.notified().await;
            continue;
        };

        let now = unix_now();
        if job.next_attempt_at > now {
            let wait = Duration::from_secs((job.next_attempt_at - now) as u64);
            tokio::select! {
                _ = sleep(wait) => {},
                _ = notify.notified() => {},
            }
            continue;
        }

        match run_job(&pool, &http, &app, &api_key, &job).await {
            Ok(()) => {}
            Err(error) => handle_failure(&pool, &job, error).await?,
        }

        sleep(Duration::from_millis(PACING_MS)).await;
    }
}

async fn run_job(
    pool: &SqlitePool,
    http: &reqwest::Client,
    app: &AppHandle,
    api_key: &str,
    job: &queries::MetadataJob,
) -> AppResult<()> {
    let posters_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| AppError::Other(format!("app_data_dir: {error}")))?
        .join("posters");

    let outcome = match job.kind.as_str() {
        "movie" => fetch_movie(pool, http, api_key, job.media_id).await?,
        "show" => fetch_show(pool, http, api_key, job.media_id).await?,
        other => {
            return Err(AppError::Other(format!("unknown job kind: {other}")));
        }
    };

    let mut tx = pool.begin().await?;
    queries::delete_in_tx(&mut *tx, &job.kind, job.media_id).await?;
    tx.commit().await?;

    if let Some((poster_url, dest_filename)) = outcome {
        let dest = posters_dir.join(dest_filename);
        // Best-effort: a failed poster download doesn't invalidate the
        // already-committed text metadata.
        if let Err(error) = tmdb::download_poster(http, &poster_url, &dest).await {
            eprintln!("poster download failed for {dest:?}: {error}");
        }
    }

    Ok(())
}

async fn fetch_movie(
    pool: &SqlitePool,
    http: &reqwest::Client,
    api_key: &str,
    movie_id: i64,
) -> AppResult<Option<(String, String)>> {
    let row: Option<(i64, String, Option<i32>)> = sqlx::query_as(
        "SELECT metadata_locked, title, year FROM movies WHERE id = ?1",
    )
    .bind(movie_id)
    .fetch_optional(pool)
    .await?;

    let Some((locked, title, year)) = row else {
        // Movie row was deleted before we got to it. Treat as success;
        // the job row gets deleted by the caller.
        return Ok(None);
    };
    if locked != 0 {
        return Ok(None);
    }

    let candidates = tmdb::search_movie(http, api_key, &title, year).await?;
    let Some(pick) = matching::pick_confident_match(&title, year, &candidates) else {
        return Ok(None);
    };

    let details = tmdb::fetch_movie_details(http, api_key, &pick.provider_id).await?;

    let mut tx = pool.begin().await?;

    // Re-check metadata_locked inside the tx to handle the race where
    // the user edited the row while we were over the wire.
    let still_locked: i64 = sqlx::query_scalar(
        "SELECT metadata_locked FROM movies WHERE id = ?1",
    )
    .bind(movie_id)
    .fetch_one(&mut *tx)
    .await?;
    if still_locked != 0 {
        tx.rollback().await?;
        return Ok(None);
    }

    let download_ext = apply::apply_movie_details(&mut *tx, movie_id, &details).await?;
    tx.commit().await?;

    Ok(match (download_ext, details.poster_path) {
        (Some(extension), Some(poster_path)) => {
            Some((poster_path, format!("movie-{movie_id}.{extension}")))
        }
        _ => None,
    })
}

async fn fetch_show(
    pool: &SqlitePool,
    http: &reqwest::Client,
    api_key: &str,
    show_id: i64,
) -> AppResult<Option<(String, String)>> {
    let row: Option<(i64, String, Option<i32>)> = sqlx::query_as(
        "SELECT metadata_locked, title, year FROM shows WHERE id = ?1",
    )
    .bind(show_id)
    .fetch_optional(pool)
    .await?;

    let Some((locked, title, year)) = row else {
        return Ok(None);
    };
    if locked != 0 {
        return Ok(None);
    }

    let candidates = tmdb::search_show(http, api_key, &title, year).await?;
    let Some(pick) = matching::pick_confident_match(&title, year, &candidates) else {
        return Ok(None);
    };

    let details = tmdb::fetch_show_details(http, api_key, &pick.provider_id).await?;

    let mut tx = pool.begin().await?;

    let still_locked: i64 = sqlx::query_scalar(
        "SELECT metadata_locked FROM shows WHERE id = ?1",
    )
    .bind(show_id)
    .fetch_one(&mut *tx)
    .await?;
    if still_locked != 0 {
        tx.rollback().await?;
        return Ok(None);
    }

    let download_ext = apply::apply_show_details(&mut *tx, show_id, &details).await?;
    tx.commit().await?;

    Ok(match (download_ext, details.poster_path) {
        (Some(extension), Some(poster_path)) => {
            Some((poster_path, format!("show-{show_id}.{extension}")))
        }
        _ => None,
    })
}

async fn handle_failure(
    pool: &SqlitePool,
    job: &queries::MetadataJob,
    error: AppError,
) -> AppResult<()> {
    let message = error.to_string();

    if message.starts_with("auth_required") {
        queries::park_auth(pool, &job.kind, job.media_id).await?;
    } else {
        queries::record_failure(pool, &job.kind, job.media_id, &message).await?;
    }

    Ok(())
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
