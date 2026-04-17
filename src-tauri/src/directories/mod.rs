use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::error::AppResult;
use crate::state::AppState;

const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "mov", "avi", "webm", "m4v"];

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Directory {
    pub id: i64,
    pub path: String,
    pub recursive: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanResult {
    pub matched: Vec<MatchedFile>,
    pub unmatched: Vec<UnmatchedFile>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchedFile {
    pub film_id: i64,
    pub file_path: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnmatchedFile {
    pub file_path: String,
    pub display_name: String,
}

fn has_video_ext(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .map(|s| VIDEO_EXTENSIONS.contains(&s.as_str()))
        .unwrap_or(false)
}

fn display_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}

#[tauri::command]
pub async fn add_directory(
    state: tauri::State<'_, AppState>,
    path: String,
    recursive: Option<bool>,
) -> AppResult<Directory> {
    let recursive = recursive.unwrap_or(true);
    let canonical = PathBuf::from(&path)
        .canonicalize()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or(path);

    let rec_int: i64 = if recursive { 1 } else { 0 };
    let row = sqlx::query_as::<_, Directory>(
        "INSERT INTO directories (path, recursive) VALUES (?, ?) \
         ON CONFLICT(path) DO UPDATE SET recursive = excluded.recursive \
         RETURNING id, path, recursive, created_at",
    )
    .bind(&canonical)
    .bind(rec_int)
    .fetch_one(&state.db)
    .await?;

    Ok(row)
}

#[tauri::command]
pub async fn list_directories(
    state: tauri::State<'_, AppState>,
) -> AppResult<Vec<Directory>> {
    let rows = sqlx::query_as::<_, Directory>(
        "SELECT id, path, recursive, created_at FROM directories ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}

#[tauri::command]
pub async fn delete_directory(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> AppResult<()> {
    let dir: Option<Directory> = sqlx::query_as(
        "SELECT id, path, recursive, created_at FROM directories WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    if let Some(dir) = dir {
        let prefix = format!("{}/%", dir.path.trim_end_matches('/'));
        sqlx::query("UPDATE films SET orphaned = 1 WHERE file_path LIKE ? OR file_path = ?")
            .bind(&prefix)
            .bind(&dir.path)
            .execute(&state.db)
            .await?;
    }

    sqlx::query("DELETE FROM directories WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn scan_directory(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> AppResult<ScanResult> {
    let dir = sqlx::query_as::<_, Directory>(
        "SELECT id, path, recursive, created_at FROM directories WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    let max_depth = if dir.recursive != 0 { usize::MAX } else { 1 };
    let root = PathBuf::from(&dir.path);

    let files: Vec<PathBuf> = tokio::task::spawn_blocking(move || {
        WalkDir::new(&root)
            .max_depth(max_depth)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| has_video_ext(e.path()))
            .map(|e| e.into_path())
            .collect()
    })
    .await
    .map_err(|e| crate::error::AppError::Other(format!("scan join error: {e}")))?;

    let mut matched = Vec::new();
    let mut unmatched = Vec::new();
    let mut found_paths: Vec<String> = Vec::with_capacity(files.len());

    for path in files {
        let path_str = path.to_string_lossy().into_owned();
        found_paths.push(path_str.clone());
        let existing: Option<(i64, String)> = sqlx::query_as(
            "SELECT id, title FROM films WHERE file_path = ?",
        )
        .bind(&path_str)
        .fetch_optional(&state.db)
        .await?;

        if let Some((film_id, title)) = existing {
            sqlx::query("UPDATE films SET orphaned = 0 WHERE id = ?")
                .bind(film_id)
                .execute(&state.db)
                .await?;
            matched.push(MatchedFile {
                film_id,
                file_path: path_str,
                title,
            });
        } else {
            let name = display_name(&path);
            unmatched.push(UnmatchedFile {
                file_path: path_str,
                display_name: name,
            });
        }
    }

    let prefix = format!("{}/%", dir.path.trim_end_matches('/'));
    let under_dir: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, file_path FROM films WHERE file_path LIKE ? OR file_path = ?",
    )
    .bind(&prefix)
    .bind(&dir.path)
    .fetch_all(&state.db)
    .await?;

    for (film_id, file_path) in under_dir {
        if !found_paths.iter().any(|p| p == &file_path) {
            sqlx::query("UPDATE films SET orphaned = 1 WHERE id = ?")
                .bind(film_id)
                .execute(&state.db)
                .await?;
        }
    }

    Ok(ScanResult { matched, unmatched })
}
