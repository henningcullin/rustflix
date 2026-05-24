use std::collections::HashSet;
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

// Files we'll accept as an auto-discovered poster, ordered by preference.
const POSTER_BASENAMES: &[&str] = &["poster", "cover", "folder"];
const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp"];

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

// Trailing season tokens we want to drop from a show name so that
// "Breaking Bad S01" / "Breaking Bad Season 02" / "Breaking Bad Series 3" all
// collapse to "Breaking Bad" for fingerprinting and display.
static STRIP_SEASON_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)[\s._-]+(?:s\d{1,2}|season[\s._-]*\d{1,3}|series[\s._-]*\d{1,3})\s*$").unwrap()
});

/// Strip a trailing `S01` / `Season 2` / `Series 3` suffix from a show name.
pub fn strip_season_suffix(name: &str) -> String {
    STRIP_SEASON_RE.replace(name.trim(), "").trim().to_string()
}

/// Stable dedup key for a show. Derived from its (already cleaned) display
/// name: any trailing season token is removed, the result is ascii-lowercased,
/// whitespace collapsed, and non-alphanumeric/whitespace characters dropped.
/// Pure function — same input always produces the same output.
pub fn fingerprint(name: &str) -> String {
    let stripped = strip_season_suffix(name);

    let lower = stripped.to_lowercase();
    let filtered: String = lower
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect();

    filtered.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_video_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| VIDEO_EXTS.iter().any(|v| v.eq_ignore_ascii_case(e)))
        .unwrap_or(false)
}

/// Escapes `%`, `_`, and `\` so a literal filename can be used inside a
/// SQL `LIKE` pattern without those characters being treated as wildcards.
/// Pair with `ESCAPE '\\'` in the query.
fn escape_like(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for character in input.chars() {
        if character == '\\' || character == '%' || character == '_' {
            output.push('\\');
        }
        output.push(character);
    }
    output
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

        let show_title = strip_season_suffix(&clean_title(&show_raw));
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

/// Look for a `poster.*`, `cover.*`, or `folder.*` image in `dir`. Preference
/// order is POSTER_BASENAMES first, then IMAGE_EXTS. Returns the first match
/// or None if the directory is unreadable / contains no candidates.
async fn find_poster_in(dir: &Path) -> Option<PathBuf> {
    let mut entries = tokio::fs::read_dir(dir).await.ok()?;

    let mut candidates: Vec<PathBuf> = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        let Ok(metadata) = entry.metadata().await else {
            continue;
        };
        if !metadata.is_file() {
            continue;
        }

        let Some(stem) = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
        else {
            continue;
        };
        let Some(ext) = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
        else {
            continue;
        };

        let is_image = IMAGE_EXTS.iter().any(|candidate| *candidate == ext);
        let is_poster_name = POSTER_BASENAMES.iter().any(|candidate| *candidate == stem);

        if is_image && is_poster_name {
            candidates.push(path);
        }
    }

    candidates.sort_by_key(|p| {
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        let basename_rank = POSTER_BASENAMES
            .iter()
            .position(|candidate| *candidate == stem)
            .unwrap_or(usize::MAX);
        let ext_rank = IMAGE_EXTS
            .iter()
            .position(|candidate| *candidate == ext)
            .unwrap_or(usize::MAX);
        (basename_rank, ext_rank)
    });

    candidates.into_iter().next()
}

/// Set `poster_path` + `poster_origin = 'auto'` on a show if and only if the
/// row doesn't have a manual poster. The search visits the show's stored
/// `folder_path` first, then every distinct parent directory of its episodes
/// — so per-season folders ("Breaking Bad S01/poster.jpg",
/// "Breaking Bad S02/poster.jpg") all get a chance to provide artwork.
async fn maybe_set_show_poster(pool: &SqlitePool, show_id: i64) -> AppResult<()> {
    let origin: Option<String> =
        sqlx::query_scalar("SELECT poster_origin FROM shows WHERE id = ?1")
            .bind(show_id)
            .fetch_one(pool)
            .await?;
    if origin.as_deref() == Some("manual") {
        return Ok(());
    }

    let folder_path: String =
        sqlx::query_scalar("SELECT folder_path FROM shows WHERE id = ?1")
            .bind(show_id)
            .fetch_one(pool)
            .await?;
    let episode_paths: Vec<String> =
        sqlx::query_scalar("SELECT DISTINCT path FROM episodes WHERE show_id = ?1")
            .bind(show_id)
            .fetch_all(pool)
            .await?;

    let mut dirs: Vec<PathBuf> = vec![PathBuf::from(&folder_path)];
    for episode_path in &episode_paths {
        if let Some(parent) = Path::new(episode_path).parent() {
            let parent_buf = parent.to_path_buf();
            if !dirs.contains(&parent_buf) {
                dirs.push(parent_buf);
            }
        }
    }

    for dir in dirs {
        if let Some(poster) = find_poster_in(&dir).await {
            let poster_str = poster.to_string_lossy().to_string();
            sqlx::query(
                "UPDATE shows SET poster_path = ?1, poster_origin = 'auto' WHERE id = ?2",
            )
            .bind(&poster_str)
            .bind(show_id)
            .execute(pool)
            .await?;
            return Ok(());
        }
    }

    Ok(())
}

/// Same as [`maybe_set_show_poster`] but for movies — scoped to the movie
/// file's parent directory only.
async fn maybe_set_movie_poster(
    pool: &SqlitePool,
    movie_id: i64,
    movie_dir: &Path,
) -> AppResult<()> {
    let origin: Option<String> =
        sqlx::query_scalar("SELECT poster_origin FROM movies WHERE id = ?1")
            .bind(movie_id)
            .fetch_one(pool)
            .await?;
    if origin.as_deref() == Some("manual") {
        return Ok(());
    }

    if let Some(poster) = find_poster_in(movie_dir).await {
        let poster_str = poster.to_string_lossy().to_string();
        sqlx::query(
            "UPDATE movies SET poster_path = ?1, poster_origin = 'auto' WHERE id = ?2",
        )
        .bind(&poster_str)
        .bind(movie_id)
        .execute(pool)
        .await?;
    }

    Ok(())
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

    // Shows we touched during this scan — used after the main loop to run
    // poster auto-discovery exactly once per show.
    let mut touched_shows: HashSet<i64> = HashSet::new();

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
                let existing_movie_id: Option<i64> =
                    sqlx::query_scalar("SELECT id FROM movies WHERE path = ?1")
                        .bind(&path_str)
                        .fetch_optional(pool)
                        .await?;

                let movie_id = match existing_movie_id {
                    Some(id) => id,
                    None => {
                        let new_id: i64 = sqlx::query_scalar(
                            "INSERT INTO movies (library_id, title, year, path)
                             VALUES (?1, ?2, ?3, ?4)
                             RETURNING id",
                        )
                        .bind(library_id)
                        .bind(&title)
                        .bind(year)
                        .bind(&path_str)
                        .fetch_one(pool)
                        .await?;

                        report.movies_added += 1;
                        crate::metadata::queries::enqueue(pool, "movie", new_id).await?;
                        new_id
                    }
                };

                if let Some(parent) = path.parent() {
                    maybe_set_movie_poster(pool, movie_id, parent).await?;
                }
            }
            Detected::Episode {
                show_title,
                show_year,
                season,
                episode,
                episode_title,
            } => {
                // If this exact file is already imported, just remember the
                // show it belongs to (so poster discovery still runs) and
                // skip every show-creation path. This is the core
                // rescan-idempotency rule.
                let existing_show_id: Option<i64> =
                    sqlx::query_scalar("SELECT show_id FROM episodes WHERE path = ?1")
                        .bind(&path_str)
                        .fetch_optional(pool)
                        .await?;

                if let Some(show_id) = existing_show_id {
                    touched_shows.insert(show_id);
                    continue;
                }

                // Resolve the show folder, but treat the library root itself
                // as "no show folder" — layouts like <library>/Season 01/file.mkv
                // would otherwise produce a folder_prefix equal to the entire
                // library and the LIKE lookup would pick the most-populated
                // show in the library by accident.
                let show_folder = find_show_folder(&path).filter(|folder| {
                    folder.as_path() != root && folder.starts_with(root)
                });

                let owning_show_id: Option<i64> = if let Some(folder) = show_folder.as_ref() {
                    let folder_str = folder.to_string_lossy().to_string();

                    // Prefer attaching new files to whatever show already owns
                    // sibling files under the same show folder. That survives a
                    // prior manual merge, where the deleted source show would
                    // otherwise be recreated by fingerprint. Escape %, _, and \
                    // so folder names containing those characters don't turn
                    // into LIKE wildcards (e.g. "100% Movies/Show/").
                    let escaped = escape_like(&folder_str);
                    let folder_prefix =
                        format!("{}{}%", escaped, std::path::MAIN_SEPARATOR);

                    sqlx::query_scalar(
                        "SELECT show_id FROM episodes
                         WHERE path LIKE ?1 ESCAPE '\\'
                         GROUP BY show_id
                         ORDER BY COUNT(*) DESC, show_id ASC
                         LIMIT 1",
                    )
                    .bind(&folder_prefix)
                    .fetch_optional(pool)
                    .await?
                } else {
                    None
                };

                let show_folder_str = show_folder
                    .as_ref()
                    .map(|folder| folder.to_string_lossy().to_string())
                    .unwrap_or_else(|| root.to_string_lossy().to_string());

                let (show_id, created_new_show) = match owning_show_id {
                    Some(id) => (id, false),
                    None => {
                        let show_fingerprint = fingerprint(&show_title);

                        // Upsert by (library_id, fingerprint) so per-season
                        // folders ("Breaking Bad S01", "Breaking Bad S02")
                        // converge to a single show row. On conflict we only
                        // refresh folder_path — user-editable fields stay.
                        let id: i64 = sqlx::query_scalar(
                            "INSERT INTO shows (library_id, title, year, folder_path, fingerprint)
                             VALUES (?1, ?2, ?3, ?4, ?5)
                             ON CONFLICT(library_id, fingerprint) DO UPDATE SET folder_path = excluded.folder_path
                             RETURNING id",
                        )
                        .bind(library_id)
                        .bind(&show_title)
                        .bind(show_year)
                        .bind(&show_folder_str)
                        .bind(&show_fingerprint)
                        .fetch_one(pool)
                        .await?;

                        let prior_count: i64 = sqlx::query_scalar(
                            "SELECT COUNT(*) FROM episodes WHERE show_id = ?1",
                        )
                        .bind(id)
                        .fetch_one(pool)
                        .await?;

                        (id, prior_count == 0)
                    }
                };

                touched_shows.insert(show_id);

                let res = sqlx::query(
                    "INSERT OR IGNORE INTO episodes (show_id, season, episode, title, path)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
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
                    if created_new_show {
                        report.shows_added += 1;
                        crate::metadata::queries::enqueue(pool, "show", show_id).await?;
                    }
                }
            }
        }
    }

    for show_id in touched_shows {
        maybe_set_show_poster(pool, show_id).await?;
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_like_passes_normal_text_through() {
        assert_eq!(escape_like("/library/Breaking Bad"), "/library/Breaking Bad");
    }

    #[test]
    fn escape_like_escapes_percent() {
        assert_eq!(escape_like("/library/100% Movies"), "/library/100\\% Movies");
    }

    #[test]
    fn escape_like_escapes_underscore() {
        assert_eq!(escape_like("/library/a_show"), "/library/a\\_show");
    }

    #[test]
    fn escape_like_escapes_backslash() {
        assert_eq!(escape_like("C:\\Media\\Show"), "C:\\\\Media\\\\Show");
    }

    #[test]
    fn escape_like_escapes_all_three_in_one_string() {
        assert_eq!(escape_like("a%b_c\\d"), "a\\%b\\_c\\\\d");
    }
}
