use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Library {
    pub id: i64,
    pub path: String,
    pub kind: LibraryKind,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum LibraryKind {
    Movies,
    Series,
    Mixed,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Movie {
    pub id: i64,
    pub title: String,
    pub year: Option<i32>,
    pub path: String,
    pub poster_path: Option<String>,
    pub poster_origin: Option<String>,
    pub overview: Option<String>,
    pub duration_seconds: Option<i64>,
    pub progress_seconds: i64,
    pub watched: bool,
    pub added_at: i64,
    pub provider: Option<String>,
    pub provider_id: Option<String>,
    pub rating: Option<f64>,
    pub genres: Option<String>,
    pub top_cast: Option<String>,
    pub runtime_minutes: Option<i64>,
    pub metadata_synced_at: Option<i64>,
    pub metadata_locked: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Show {
    pub id: i64,
    pub library_id: i64,
    pub title: String,
    pub year: Option<i32>,
    pub folder_path: String,
    pub fingerprint: String,
    pub poster_path: Option<String>,
    pub poster_origin: Option<String>,
    pub overview: Option<String>,
    pub episode_count: i64,
    pub watched_count: i64,
    pub added_at: i64,
    pub provider: Option<String>,
    pub provider_id: Option<String>,
    pub rating: Option<f64>,
    pub genres: Option<String>,
    pub top_cast: Option<String>,
    pub first_air_date: Option<String>,
    pub metadata_synced_at: Option<i64>,
    pub metadata_locked: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Episode {
    pub id: i64,
    pub show_id: i64,
    pub season: i32,
    pub episode: i32,
    pub title: String,
    pub path: String,
    pub duration_seconds: Option<i64>,
    pub progress_seconds: i64,
    pub watched: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Season {
    pub season: i32,
    pub episodes: Vec<Episode>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ContinueWatchingItem {
    Movie { movie: Movie },
    Episode { show: Show, episode: Episode },
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ScanReport {
    pub libraries_scanned: usize,
    pub movies_added: usize,
    pub episodes_added: usize,
    pub shows_added: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpisodeRef {
    pub season: i32,
    pub episode: i32,
}

/// Result of a merge attempt. If `conflicts` is empty, the merge succeeded.
/// Otherwise the source show still exists and the listed episodes are the
/// (season, episode) pairs that exist in both target and source — the user
/// has to resolve them before the merge can complete.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MergeOutcome {
    pub conflicts: Vec<EpisodeRef>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MetadataStatusCounts {
    pub pending: i64,
    pub failed: i64,
    pub auth_required: i64,
    pub dead_letter: i64,
    pub needs_review: i64,
}
