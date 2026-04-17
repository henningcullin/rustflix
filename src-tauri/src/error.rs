use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("tmdb api key is not set")]
    MissingTmdbKey,

    #[error("not found")]
    NotFound,

    #[error("invalid range: {0}")]
    InvalidRange(String),

    #[error("{0}")]
    Other(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
