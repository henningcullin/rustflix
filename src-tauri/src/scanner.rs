use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use regex::Regex;
use sqlx::SqlitePool;
use walkdir::WalkDir;

use crate::error::AppResult;
use crate::models::{LibraryKind, ScanReport};

const VIDEO_EXTS: &[&str] = &[
    "mkv", "mp4", "avi", "m4v", "webm", "mov", "ts", "wmv", "flv", "mpg", "mpeg",
];

static EPISODE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:s|season[\s._-]*)(\d{1,2})[\s._-]*(?:e|x|episode[\s._-]*)(\d{1,3})").unwrap()
});

static YEAR_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:\(|\[|\.|\s|-|_)(19\d{2}|20\d{2})(?:\)|\]|\.|\s|-|_|$)").unwrap()
});

static TAGS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(1080p|2160p|720p|480p|4k|uhd|hdr|bluray|blu-ray|brrip|bdrip|webrip|web-dl|webdl|hdtv|dvdrip|x264|x265|hevc|h264|h265|aac|ac3|dts|10bit|hdr10|atmos|truehd|remux|imax|repack|proper|extended|directors\.?cut|unrated|amzn|nf|hulu|dsnp|disney\+?)\b"
    ).unwrap()
});

static SEASON_FOLDER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^season[\s._-]*\d+$|^s\d+$").unwrap());

fn is_video_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| VIDEO_EXTS.iter().any(|v| v.eq_ignore_ascii_case(e)))
        .unwrap_or(false)
}

fn clean_title(raw: &str) -> String {
    let no_ext = raw.rsplit_once('.').map(|(a, _)| a).unwrap_or(raw);
    let lower_tags = TAGS_RE.replace_all(no_ext, " ");
    let cut = if let Some(m) = YEAR_RE.find(&lower_tags) {
        lower_tags[..m.start()].to_string()
    } else {
        lower_tags.to_string()
    };
    cut.replace('.', " ")
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches(|c: char| c == '-' || c == ' ' || c == '(' || c == ')' || c == '[' || c == ']')
        .to_string()
}

fn extract_year(raw: &str) -> Option<i32> {
    YEAR_RE
        .captures(raw)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

#[derive(Debug)]
enum Detected {
    Movie {
        title: String,
        year: Option<i32>,
    },
    Episode {
        show_title: String,
        show_year: Option<i32>,
        season: i32,
        episode: i32,
        episode_title: String,
    },
}

fn detect(path: &Path, hint: LibraryKind) -> Option<Detected> {
    let file_name = path.file_name()?.to_string_lossy().to_string();
    let parent_name = path
        .parent()
        .and_then(|p| p.file_name())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let ep_match = EPISODE_RE.captures(&file_name);

    let treat_as_episode = match hint {
        LibraryKind::Movies => false,
        LibraryKind::Series => true,
        LibraryKind::Mixed => ep_match.is_some(),
    };

    if treat_as_episode {
        let caps = ep_match.or_else(|| EPISODE_RE.captures(&parent_name))?;
        let season: i32 = caps.get(1)?.as_str().parse().ok()?;
        let episode: i32 = caps.get(2)?.as_str().parse().ok()?;

        let mut show_dir: Option<&Path> = path.parent();
        while let Some(dir) = show_dir {
            let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.is_empty() && !SEASON_FOLDER_RE.is_match(name) {
                break;
            }
            show_dir = dir.parent();
        }
        let show_raw = show_dir
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| parent_name.clone());

        let show_title = clean_title(&show_raw);
        let show_year = extract_year(&show_raw);

        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or(file_name.clone());
        let after = EPISODE_RE
            .find(&stem)
            .map(|m| stem[m.end()..].to_string())
            .unwrap_or_default();
        let cleaned = clean_title(after.trim_matches(|c: char| !c.is_alphanumeric()));
        let episode_title = if cleaned.is_empty() {
            format!("Episode {}", episode)
        } else {
            cleaned
        };

        Some(Detected::Episode {
            show_title,
            show_year,
            season,
            episode,
            episode_title,
        })
    } else {
        Some(Detected::Movie {
            title: clean_title(&file_name),
            year: extract_year(&file_name),
        })
    }
}

fn find_show_folder(episode_path: &Path) -> Option<PathBuf> {
    let mut current = episode_path.parent()?;
    loop {
        let name = current.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !SEASON_FOLDER_RE.is_match(name) {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
}

pub async fn scan_library(
    pool: &SqlitePool,
    library_id: i64,
    root: &Path,
    kind: LibraryKind,
) -> AppResult<ScanReport> {
    let mut report = ScanReport {
        libraries_scanned: 1,
        ..Default::default()
    };

    // Enumerate files off the async runtime to avoid blocking executor threads.
    let root_owned = root.to_path_buf();
    let paths: Vec<PathBuf> = tokio::task::spawn_blocking(move || {
        WalkDir::new(&root_owned)
            .follow_links(true)
            .into_iter()
            .flatten()
            .filter(|e| e.file_type().is_file())
            .map(|e| e.into_path())
            .filter(|p| is_video_file(p))
            .collect()
    })
    .await
    .map_err(|e| crate::error::AppError::Other(e.to_string()))?;

    for path in paths {
        let path_str = path.to_string_lossy().to_string();
        let Some(detected) = detect(&path, kind) else { continue };

        match detected {
            Detected::Movie { title, year } => {
                let res = sqlx::query(
                    "INSERT OR IGNORE INTO movies (library_id, title, year, path) VALUES (?1, ?2, ?3, ?4)",
                )
                .bind(library_id)
                .bind(&title)
                .bind(year)
                .bind(&path_str)
                .execute(pool)
                .await?;
                if res.rows_affected() > 0 {
                    report.movies_added += 1;
                }
            }
            Detected::Episode {
                show_title,
                show_year,
                season,
                episode,
                episode_title,
            } => {
                let show_folder = find_show_folder(&path).unwrap_or_else(|| root.to_path_buf());
                let show_folder_str = show_folder.to_string_lossy().to_string();

                // Upsert show; ON CONFLICT keeps existing fields, returns id either way.
                let show_id: i64 = sqlx::query_scalar(
                    "INSERT INTO shows (library_id, title, year, folder_path) VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(folder_path) DO UPDATE SET folder_path = excluded.folder_path
                     RETURNING id",
                )
                .bind(library_id)
                .bind(&show_title)
                .bind(show_year)
                .bind(&show_folder_str)
                .fetch_one(pool)
                .await?;

                let prior_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM episodes WHERE show_id = ?1",
                )
                .bind(show_id)
                .fetch_one(pool)
                .await?;

                let res = sqlx::query(
                    "INSERT OR IGNORE INTO episodes (show_id, season, episode, title, path) VALUES (?1, ?2, ?3, ?4, ?5)",
                )
                .bind(show_id)
                .bind(season)
                .bind(episode)
                .bind(&episode_title)
                .bind(&path_str)
                .execute(pool)
                .await?;

                if res.rows_affected() > 0 {
                    report.episodes_added += 1;
                    if prior_count == 0 {
                        report.shows_added += 1;
                    }
                }
            }
        }
    }

    Ok(report)
}
