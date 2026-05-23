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
pub async fn merge_shows(
    db: State<'_, Db>,
    target_id: i64,
    source_id: i64,
) -> AppResult<MergeOutcome> {
    queries::merge_shows(&db, target_id, source_id).await
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
