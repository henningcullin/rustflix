//! Background worker that drains `metadata_jobs`. One task, sequential,
//! 250ms pacing between requests. Waits on a Notify when the queue is
//! empty, when there's no API key, or when every job is parked on
//! auth_required.
//!
//! Per job: read `metadata_mode`, ask `providers_for_mode` for the walk,
//! call each provider with 250ms intra-walk pacing, then classify the
//! end-of-walk outcome (matched / saw_tmdb_auth → park / has_error →
//! backoff / no_error → delete).

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};
use tokio::sync::Notify;
use tokio::time::sleep;

use crate::error::{AppError, AppResult};
use crate::metadata::dispatch::{providers_for_mode, ParkReason, Provider};
use crate::metadata::{apply, imdb, matching, queries, tmdb};
use crate::queries as app_queries;

const PACING_MS: u64 = 250;

/// Mirrors the `attempts < 8` filter inside `queries::next_due`. Kept
/// in worker code so the loop can short-circuit before doing any work
/// for a job that should already be off the queue.
const MAX_ATTEMPTS: i64 = 8;

/// Outcome of attempting one provider against one job.
enum Outcome {
    Matched,
    NoMatch,
}

/// A pending poster download, queued post-tx as best-effort. `size` is
/// the discriminator: `None` means "TMDB path, route through
/// `tmdb::download_poster`"; `Some(PosterSize)` means "IMDB path, route
/// through `imdb::download_poster`".
pub struct PosterDownload {
    pub url: String,
    pub filename: String,
    pub size: Option<crate::metadata::imdb::PosterSize>,
}

/// Match result from a per-kind dispatcher (e.g. `dispatch_tmdb_movie`).
enum MatchOutcome {
    NoMatch,
    Matched { poster: Option<PosterDownload> },
}

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
        let mode = app_queries::get_app_setting(&pool, "metadata_mode")
            .await?
            .unwrap_or_else(|| "prefer_tmdb".to_string());

        let api_key = app_queries::get_app_setting(&pool, "tmdb_api_key").await?;
        let key_at_job_start = api_key.clone();

        let Some(job) = queries::next_due(&pool).await? else {
            notify.notified().await;
            continue;
        };

        if job.attempts >= MAX_ATTEMPTS {
            sleep(Duration::from_secs(60)).await;
            continue;
        }

        let now = unix_now();
        if job.next_attempt_at > now {
            let wait = Duration::from_secs((job.next_attempt_at - now) as u64);
            tokio::select! {
                _ = sleep(wait) => {},
                _ = notify.notified() => {},
            }
            continue;
        }

        let providers = match providers_for_mode(&mode, api_key.is_some()) {
            Ok(list) => list,
            Err(reason) => {
                queries::park_with_reason(&pool, &job.kind, job.media_id, reason).await?;
                continue;
            }
        };

        if providers.is_empty() {
            // mode == "off" — drain the queue.
            let mut tx = pool.begin().await?;
            queries::delete_in_tx(&mut *tx, &job.kind, job.media_id).await?;
            tx.commit().await?;
            continue;
        }

        let mut last_err: Option<AppError> = None;
        let mut saw_tmdb_auth = false;
        let mut matched = false;

        for (index, provider) in providers.iter().enumerate() {
            if index > 0 {
                sleep(Duration::from_millis(PACING_MS)).await;
            }

            let key_for_call = api_key.as_deref().unwrap_or("");
            match dispatch_provider(*provider, &pool, &http, &app, key_for_call, &job).await {
                Ok(Outcome::Matched) => {
                    matched = true;
                    break;
                }
                Ok(Outcome::NoMatch) => continue,
                Err(error) => {
                    let error_string = error.to_string();
                    if error_string.starts_with("auth_required")
                        || error_string.starts_with("tmdb_auth_required")
                    {
                        saw_tmdb_auth = true;
                        let key_now =
                            app_queries::get_app_setting(&pool, "tmdb_api_key").await?;
                        if key_now == key_at_job_start {
                            app_queries::set_app_setting(&pool, "tmdb_auth_bad", "1").await?;
                        }
                    }
                    last_err = Some(error);
                    continue;
                }
            }
        }

        if matched {
            // dispatch_tmdb_* already committed apply + delete_in_tx in one tx.
        } else if saw_tmdb_auth {
            queries::park_with_reason(
                &pool,
                &job.kind,
                job.media_id,
                ParkReason::TmdbAuthRequired,
            )
            .await?;
        } else if let Some(error) = last_err {
            queries::record_failure(&pool, &job.kind, job.media_id, &error.to_string()).await?;
        } else {
            let mut tx = pool.begin().await?;
            queries::delete_in_tx(&mut *tx, &job.kind, job.media_id).await?;
            tx.commit().await?;
        }

        sleep(Duration::from_millis(PACING_MS)).await;
    }
}

async fn dispatch_provider(
    provider: Provider,
    pool: &SqlitePool,
    http: &reqwest::Client,
    app: &AppHandle,
    api_key: &str,
    job: &queries::MetadataJob,
) -> AppResult<Outcome> {
    match provider {
        Provider::Tmdb => dispatch_tmdb(pool, http, app, api_key, job).await,
        Provider::Imdb => dispatch_imdb(pool, http, app, job).await,
    }
}

async fn dispatch_tmdb(
    pool: &SqlitePool,
    http: &reqwest::Client,
    app: &AppHandle,
    api_key: &str,
    job: &queries::MetadataJob,
) -> AppResult<Outcome> {
    let posters_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| AppError::Other(format!("app_data_dir: {error}")))?
        .join("posters");

    let result = match job.kind.as_str() {
        "movie" => dispatch_tmdb_movie(pool, http, api_key, job.media_id).await?,
        "show" => dispatch_tmdb_show(pool, http, api_key, job.media_id).await?,
        other => {
            return Err(AppError::Other(format!("unknown job kind: {other}")));
        }
    };

    match result {
        MatchOutcome::NoMatch => Ok(Outcome::NoMatch),
        MatchOutcome::Matched { poster } => {
            if let Some(download) = poster {
                let dest = posters_dir.join(&download.filename);
                // Best-effort: a failed poster download doesn't invalidate
                // the already-committed text metadata.
                if let Err(error) = tmdb::download_poster(http, &download.url, &dest).await {
                    eprintln!("tmdb poster download failed for {dest:?}: {error}");
                }
            }
            Ok(Outcome::Matched)
        }
    }
}

async fn dispatch_tmdb_movie(
    pool: &SqlitePool,
    http: &reqwest::Client,
    api_key: &str,
    movie_id: i64,
) -> AppResult<MatchOutcome> {
    let row: Option<(i64, String, Option<i32>)> = sqlx::query_as(
        "SELECT metadata_locked, title, year FROM movies WHERE id = ?1",
    )
    .bind(movie_id)
    .fetch_optional(pool)
    .await?;

    let Some((locked, title, year)) = row else {
        return Ok(MatchOutcome::NoMatch);
    };
    if locked != 0 {
        return Ok(MatchOutcome::NoMatch);
    }

    let candidates = tmdb::search_movie(http, api_key, &title, year).await?;
    let Some(pick) = matching::pick_confident_match(&title, year, &candidates) else {
        return Ok(MatchOutcome::NoMatch);
    };

    let details = tmdb::fetch_movie_details(http, api_key, &pick.provider_id).await?;

    let mut tx = pool.begin().await?;

    // Re-check the lock inside the tx — the user may have edited the
    // title while we were over the wire.
    let still_locked: i64 = sqlx::query_scalar(
        "SELECT metadata_locked FROM movies WHERE id = ?1",
    )
    .bind(movie_id)
    .fetch_one(&mut *tx)
    .await?;
    if still_locked != 0 {
        tx.rollback().await?;
        return Ok(MatchOutcome::NoMatch);
    }

    let download_ext = apply::apply_movie_details(&mut *tx, movie_id, &details).await?;

    // Merged delete: apply + delete go in one tx so a concurrent
    // re-enqueue between the two writes can't be silently dropped.
    queries::delete_in_tx(&mut *tx, "movie", movie_id).await?;
    tx.commit().await?;

    Ok(MatchOutcome::Matched {
        poster: match (download_ext, details.poster_path) {
            (Some(extension), Some(poster_path)) => Some(PosterDownload {
                url: poster_path,
                filename: format!("movie-{movie_id}.{extension}"),
                size: None,
            }),
            _ => None,
        },
    })
}

async fn dispatch_tmdb_show(
    pool: &SqlitePool,
    http: &reqwest::Client,
    api_key: &str,
    show_id: i64,
) -> AppResult<MatchOutcome> {
    let row: Option<(i64, String, Option<i32>)> = sqlx::query_as(
        "SELECT metadata_locked, title, year FROM shows WHERE id = ?1",
    )
    .bind(show_id)
    .fetch_optional(pool)
    .await?;

    let Some((locked, title, year)) = row else {
        return Ok(MatchOutcome::NoMatch);
    };
    if locked != 0 {
        return Ok(MatchOutcome::NoMatch);
    }

    let candidates = tmdb::search_show(http, api_key, &title, year).await?;
    let Some(pick) = matching::pick_confident_match(&title, year, &candidates) else {
        return Ok(MatchOutcome::NoMatch);
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
        return Ok(MatchOutcome::NoMatch);
    }

    let download_ext = apply::apply_show_details(&mut *tx, show_id, &details).await?;

    queries::delete_in_tx(&mut *tx, "show", show_id).await?;
    tx.commit().await?;

    Ok(MatchOutcome::Matched {
        poster: match (download_ext, details.poster_path) {
            (Some(extension), Some(poster_path)) => Some(PosterDownload {
                url: poster_path,
                filename: format!("show-{show_id}.{extension}"),
                size: None,
            }),
            _ => None,
        },
    })
}

async fn dispatch_imdb(
    pool: &SqlitePool,
    http: &reqwest::Client,
    app: &AppHandle,
    job: &queries::MetadataJob,
) -> AppResult<Outcome> {
    let posters_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| AppError::Other(format!("app_data_dir: {error}")))?
        .join("posters");

    let result = match job.kind.as_str() {
        "movie" => dispatch_imdb_movie(pool, http, job.media_id).await?,
        "show" => dispatch_imdb_show(pool, http, job.media_id).await?,
        other => {
            return Err(AppError::Other(format!("unknown job kind: {other}")));
        }
    };

    match result {
        MatchOutcome::NoMatch => Ok(Outcome::NoMatch),
        MatchOutcome::Matched { poster } => {
            if let Some(download) = poster {
                let dest = posters_dir.join(&download.filename);
                let size = download
                    .size
                    .unwrap_or(crate::metadata::imdb::PosterSize::Small);
                if let Err(error) = imdb::download_poster(http, &download.url, &dest, size).await {
                    eprintln!("imdb poster download failed for {dest:?}: {error}");
                }
            }
            Ok(Outcome::Matched)
        }
    }
}

async fn dispatch_imdb_movie(
    pool: &SqlitePool,
    http: &reqwest::Client,
    movie_id: i64,
) -> AppResult<MatchOutcome> {
    let row: Option<(i64, String, Option<i32>)> = sqlx::query_as(
        "SELECT metadata_locked, title, year FROM movies WHERE id = ?1",
    )
    .bind(movie_id)
    .fetch_optional(pool)
    .await?;

    let Some((locked, title, year)) = row else {
        return Ok(MatchOutcome::NoMatch);
    };
    if locked != 0 {
        return Ok(MatchOutcome::NoMatch);
    }

    let candidates = imdb::search_movie(http, &title, year).await?;
    let Some(pick) = matching::pick_confident_match(&title, year, &candidates) else {
        return Ok(MatchOutcome::NoMatch);
    };

    let details = imdb::fetch_movie_details(http, &pick.provider_id).await?;

    let mut tx = pool.begin().await?;

    let still_locked: i64 = sqlx::query_scalar(
        "SELECT metadata_locked FROM movies WHERE id = ?1",
    )
    .bind(movie_id)
    .fetch_one(&mut *tx)
    .await?;
    if still_locked != 0 {
        tx.rollback().await?;
        return Ok(MatchOutcome::NoMatch);
    }

    let download_target = apply::apply_imdb_movie_details(&mut *tx, movie_id, &details).await?;

    queries::delete_in_tx(&mut *tx, "movie", movie_id).await?;
    tx.commit().await?;

    Ok(MatchOutcome::Matched {
        poster: download_target.map(|(size, url, filename)| PosterDownload {
            url,
            filename,
            size: Some(size),
        }),
    })
}

async fn dispatch_imdb_show(
    pool: &SqlitePool,
    http: &reqwest::Client,
    show_id: i64,
) -> AppResult<MatchOutcome> {
    let row: Option<(i64, String, Option<i32>)> = sqlx::query_as(
        "SELECT metadata_locked, title, year FROM shows WHERE id = ?1",
    )
    .bind(show_id)
    .fetch_optional(pool)
    .await?;

    let Some((locked, title, year)) = row else {
        return Ok(MatchOutcome::NoMatch);
    };
    if locked != 0 {
        return Ok(MatchOutcome::NoMatch);
    }

    let candidates = imdb::search_show(http, &title, year).await?;
    let Some(pick) = matching::pick_confident_match(&title, year, &candidates) else {
        return Ok(MatchOutcome::NoMatch);
    };

    let details = imdb::fetch_show_details(http, &pick.provider_id).await?;

    let mut tx = pool.begin().await?;

    let still_locked: i64 = sqlx::query_scalar(
        "SELECT metadata_locked FROM shows WHERE id = ?1",
    )
    .bind(show_id)
    .fetch_one(&mut *tx)
    .await?;
    if still_locked != 0 {
        tx.rollback().await?;
        return Ok(MatchOutcome::NoMatch);
    }

    let download_target = apply::apply_imdb_show_details(&mut *tx, show_id, &details).await?;

    queries::delete_in_tx(&mut *tx, "show", show_id).await?;
    tx.commit().await?;

    Ok(MatchOutcome::Matched {
        poster: download_target.map(|(size, url, filename)| PosterDownload {
            url,
            filename,
            size: Some(size),
        }),
    })
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
