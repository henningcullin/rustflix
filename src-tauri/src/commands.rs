use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager, State};

use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::models::{
    ContinueWatchingItem, Episode, Library, LibraryKind, MergeOutcome, Movie, ScanReport, Season,
    Show,
};
use crate::{player, queries, scanner};

const ALLOWED_POSTER_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp"];

#[tauri::command]
pub async fn list_libraries(db: State<'_, Db>) -> AppResult<Vec<Library>> {
    queries::list_libraries(&db).await
}

#[tauri::command]
pub async fn add_library(
    db: State<'_, Db>,
    path: String,
    kind: Option<String>,
) -> AppResult<Library> {
    let kind = match kind.as_deref() {
        Some("movies") => LibraryKind::Movies,
        Some("series") => LibraryKind::Series,
        _ => LibraryKind::Mixed,
    };
    let root = PathBuf::from(&path);
    if !root.exists() {
        return Err(AppError::Other(format!("path does not exist: {path}")));
    }
    let lib = queries::add_library(&db, &path, kind).await?;
    scanner::scan_library(&db, lib.id, &root, kind).await?;
    Ok(lib)
}

#[tauri::command]
pub async fn remove_library(db: State<'_, Db>, id: i64) -> AppResult<()> {
    queries::remove_library(&db, id).await
}

#[tauri::command]
pub async fn scan_libraries(db: State<'_, Db>) -> AppResult<ScanReport> {
    let libs = queries::list_libraries(&db).await?;
    let mut report = ScanReport::default();
    for lib in libs {
        let root = PathBuf::from(&lib.path);
        if !root.exists() {
            continue;
        }
        let r = scanner::scan_library(&db, lib.id, &root, lib.kind).await?;
        report.libraries_scanned += 1;
        report.movies_added += r.movies_added;
        report.episodes_added += r.episodes_added;
        report.shows_added += r.shows_added;
    }
    Ok(report)
}

#[tauri::command]
pub async fn list_movies(db: State<'_, Db>) -> AppResult<Vec<Movie>> {
    queries::list_movies(&db).await
}

#[tauri::command]
pub async fn get_movie(db: State<'_, Db>, id: i64) -> AppResult<Movie> {
    queries::get_movie(&db, id).await
}

#[tauri::command]
pub async fn list_shows(db: State<'_, Db>) -> AppResult<Vec<Show>> {
    queries::list_shows(&db).await
}

#[tauri::command]
pub async fn get_show(db: State<'_, Db>, id: i64) -> AppResult<Show> {
    queries::get_show(&db, id).await
}

#[tauri::command]
pub async fn get_seasons(db: State<'_, Db>, show_id: i64) -> AppResult<Vec<Season>> {
    queries::list_seasons(&db, show_id).await
}

#[tauri::command]
pub async fn get_episode(db: State<'_, Db>, id: i64) -> AppResult<Episode> {
    queries::get_episode(&db, id).await
}

#[tauri::command]
pub async fn continue_watching(db: State<'_, Db>) -> AppResult<Vec<ContinueWatchingItem>> {
    queries::continue_watching(&db, 12).await
}

#[tauri::command]
pub async fn set_watched(
    db: State<'_, Db>,
    kind: String,
    id: i64,
    watched: bool,
) -> AppResult<()> {
    let kind_str = match kind.as_str() {
        "movie" => "movie",
        "episode" => "episode",
        _ => return Err(AppError::Other(format!("unknown media kind: {kind}"))),
    };
    queries::upsert_progress(&db, kind_str, id, 0, None, watched).await
}

#[tauri::command]
pub async fn check_mpv(_db: State<'_, Db>) -> AppResult<bool> {
    Ok(player::check_mpv().await.is_ok())
}

#[tauri::command]
pub async fn play_movie(
    db: State<'_, Db>,
    id: i64,
    resume: Option<i64>,
) -> AppResult<player::PlayResult> {
    let movie = queries::get_movie(&db, id).await?;
    let start = resume.unwrap_or(movie.progress_seconds);
    player::play(&db, "movie", id, &movie.path, start).await
}

#[tauri::command]
pub async fn play_episode(
    db: State<'_, Db>,
    id: i64,
    resume: Option<i64>,
) -> AppResult<player::PlayResult> {
    let ep = queries::get_episode(&db, id).await?;
    let start = resume.unwrap_or(ep.progress_seconds);
    player::play(&db, "episode", id, &ep.path, start).await
}

#[tauri::command]
pub async fn update_show_metadata(
    db: State<'_, Db>,
    id: i64,
    title: Option<String>,
    year: Option<i32>,
    overview: Option<String>,
) -> AppResult<Show> {
    queries::update_show_metadata(&db, id, title.as_deref(), year, overview.as_deref()).await?;
    queries::get_show(&db, id).await
}

#[tauri::command]
pub async fn update_movie_metadata(
    db: State<'_, Db>,
    id: i64,
    title: Option<String>,
    year: Option<i32>,
    overview: Option<String>,
) -> AppResult<Movie> {
    queries::update_movie_metadata(&db, id, title.as_deref(), year, overview.as_deref()).await?;
    queries::get_movie(&db, id).await
}

#[tauri::command]
pub async fn update_episode_title(
    db: State<'_, Db>,
    id: i64,
    title: String,
) -> AppResult<Episode> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(AppError::Other("episode title cannot be empty".to_string()));
    }

    queries::update_episode_title(&db, id, trimmed).await?;
    queries::get_episode(&db, id).await
}

#[tauri::command]
pub async fn merge_shows(
    db: State<'_, Db>,
    target_id: i64,
    source_id: i64,
) -> AppResult<MergeOutcome> {
    queries::merge_shows(&db, target_id, source_id).await
}

#[tauri::command]
pub async fn delete_show(app: AppHandle, db: State<'_, Db>, id: i64) -> AppResult<()> {
    let show = queries::get_show(&db, id).await?;

    queries::delete_show(&db, id).await?;

    if show.poster_origin.as_deref() == Some("manual") {
        if let Some(poster_path) = show.poster_path.as_deref() {
            try_remove_managed_poster(&app, poster_path).await;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_tmdb_api_key(db: State<'_, Db>) -> AppResult<Option<String>> {
    queries::get_app_setting(&db, "tmdb_api_key").await
}

#[tauri::command]
pub async fn set_tmdb_api_key(db: State<'_, Db>, key: String) -> AppResult<()> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        queries::delete_app_setting(&db, "tmdb_api_key").await
    } else {
        queries::set_app_setting(&db, "tmdb_api_key", trimmed).await
    }
}

#[tauri::command]
pub async fn metadata_status_counts(
    db: State<'_, Db>,
) -> AppResult<crate::models::MetadataStatusCounts> {
    queries::metadata_status_counts(&db).await
}

#[tauri::command]
pub async fn fetch_metadata_now(
    app: AppHandle,
    db: State<'_, Db>,
    http: State<'_, reqwest::Client>,
    kind: String,
    id: i64,
) -> AppResult<()> {
    let api_key = queries::get_app_setting(&db, "tmdb_api_key")
        .await?
        .ok_or_else(|| AppError::Other("auth_required: no TMDB key configured".to_string()))?;

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| AppError::Other(format!("app_data_dir: {error}")))?;
    let posters_dir = app_data_dir.join("posters");

    let outcome = match kind.as_str() {
        "movie" => fetch_one_movie(&db, &http, &api_key, id).await?,
        "show" => fetch_one_show(&db, &http, &api_key, id).await?,
        other => {
            return Err(AppError::Other(format!("unknown kind: {other}")));
        }
    };

    if let Some((poster_url, dest_filename)) = outcome {
        let dest = posters_dir.join(dest_filename);
        crate::metadata::tmdb::download_poster(&http, &poster_url, &dest).await?;
    }

    Ok(())
}

async fn fetch_one_movie(
    db: &Db,
    http: &reqwest::Client,
    api_key: &str,
    movie_id: i64,
) -> AppResult<Option<(String, String)>> {
    let mut tx = db.begin().await?;

    let locked: i64 = sqlx::query_scalar("SELECT metadata_locked FROM movies WHERE id = ?1")
        .bind(movie_id)
        .fetch_one(&mut *tx)
        .await?;
    if locked != 0 {
        tx.rollback().await?;
        return Ok(None);
    }

    let (title, year): (String, Option<i32>) =
        sqlx::query_as("SELECT title, year FROM movies WHERE id = ?1")
            .bind(movie_id)
            .fetch_one(&mut *tx)
            .await?;
    tx.rollback().await?;

    let candidates = crate::metadata::tmdb::search_movie(http, api_key, &title, year).await?;
    let Some(pick) =
        crate::metadata::matching::pick_confident_match(&title, year, &candidates)
    else {
        return Ok(None);
    };

    let details =
        crate::metadata::tmdb::fetch_movie_details(http, api_key, &pick.provider_id).await?;

    let mut tx = db.begin().await?;
    let download_ext =
        crate::metadata::apply::apply_movie_details(&mut *tx, movie_id, &details).await?;
    tx.commit().await?;

    Ok(match (download_ext, details.poster_path) {
        (Some(extension), Some(poster_path)) => {
            Some((poster_path, format!("movie-{movie_id}.{extension}")))
        }
        _ => None,
    })
}

async fn fetch_one_show(
    db: &Db,
    http: &reqwest::Client,
    api_key: &str,
    show_id: i64,
) -> AppResult<Option<(String, String)>> {
    let mut tx = db.begin().await?;

    let locked: i64 = sqlx::query_scalar("SELECT metadata_locked FROM shows WHERE id = ?1")
        .bind(show_id)
        .fetch_one(&mut *tx)
        .await?;
    if locked != 0 {
        tx.rollback().await?;
        return Ok(None);
    }

    let (title, year): (String, Option<i32>) =
        sqlx::query_as("SELECT title, year FROM shows WHERE id = ?1")
            .bind(show_id)
            .fetch_one(&mut *tx)
            .await?;
    tx.rollback().await?;

    let candidates = crate::metadata::tmdb::search_show(http, api_key, &title, year).await?;
    let Some(pick) =
        crate::metadata::matching::pick_confident_match(&title, year, &candidates)
    else {
        return Ok(None);
    };

    let details =
        crate::metadata::tmdb::fetch_show_details(http, api_key, &pick.provider_id).await?;

    let mut tx = db.begin().await?;
    let download_ext =
        crate::metadata::apply::apply_show_details(&mut *tx, show_id, &details).await?;
    tx.commit().await?;

    Ok(match (download_ext, details.poster_path) {
        (Some(extension), Some(poster_path)) => {
            Some((poster_path, format!("show-{show_id}.{extension}")))
        }
        _ => None,
    })
}

/// Best-effort cleanup of a manual poster file. Only deletes the file when
/// it sits inside our own `<app_data>/posters/` directory — any other path
/// is ignored so a malformed `poster_path` can never remove user media.
async fn try_remove_managed_poster(app: &AppHandle, poster_path: &str) {
    let app_data_dir = match app.path().app_data_dir() {
        Ok(dir) => dir,
        Err(_) => return,
    };
    let posters_dir = app_data_dir.join("posters");

    let candidate = Path::new(poster_path);
    if !candidate.starts_with(&posters_dir) {
        return;
    }

    let _ = tokio::fs::remove_file(candidate).await;
}

#[tauri::command]
pub async fn set_show_poster_from_file(
    app: AppHandle,
    db: State<'_, Db>,
    id: i64,
    source_path: String,
) -> AppResult<Show> {
    let destination = copy_poster(&app, "show", id, &source_path).await?;
    queries::set_show_poster(&db, id, &destination, "manual").await?;
    queries::get_show(&db, id).await
}

#[tauri::command]
pub async fn set_movie_poster_from_file(
    app: AppHandle,
    db: State<'_, Db>,
    id: i64,
    source_path: String,
) -> AppResult<Movie> {
    let destination = copy_poster(&app, "movie", id, &source_path).await?;
    queries::set_movie_poster(&db, id, &destination, "manual").await?;
    queries::get_movie(&db, id).await
}

#[tauri::command]
pub async fn reset_show_poster(db: State<'_, Db>, id: i64) -> AppResult<Show> {
    queries::reset_show_poster(&db, id).await?;
    queries::get_show(&db, id).await
}

#[tauri::command]
pub async fn reset_movie_poster(db: State<'_, Db>, id: i64) -> AppResult<Movie> {
    queries::reset_movie_poster(&db, id).await?;
    queries::get_movie(&db, id).await
}

/// Copy `source_path` into `<app_data>/posters/{kind}-{id}.{ext}` and return
/// the destination path as a string. Rejects sources whose extension isn't
/// in [`ALLOWED_POSTER_EXTS`] so we don't accidentally accept arbitrary
/// files.
async fn copy_poster(
    app: &AppHandle,
    kind: &str,
    id: i64,
    source_path: &str,
) -> AppResult<String> {
    let source = Path::new(source_path);
    if !source.exists() {
        return Err(AppError::Other(format!(
            "source file does not exist: {source_path}"
        )));
    }

    let extension = source
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .ok_or_else(|| AppError::Other("source file has no extension".to_string()))?;
    if !ALLOWED_POSTER_EXTS.iter().any(|allowed| *allowed == extension) {
        return Err(AppError::Other(format!(
            "unsupported image type: .{extension}"
        )));
    }

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| AppError::Other(format!("app_data_dir: {error}")))?;
    let poster_dir = app_data_dir.join("posters");
    tokio::fs::create_dir_all(&poster_dir).await?;

    let destination = poster_dir.join(format!("{kind}-{id}.{extension}"));
    tokio::fs::copy(source, &destination).await?;

    Ok(destination.to_string_lossy().to_string())
}
