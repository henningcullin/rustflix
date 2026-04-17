use std::path::PathBuf;
use std::sync::Arc;

use sqlx::SqlitePool;
use tokio::sync::RwLock;

use crate::settings::SettingsStore;
use crate::tmdb::TmdbClient;

pub struct AppState {
    pub db: SqlitePool,
    pub app_data_dir: PathBuf,
    pub settings: Arc<RwLock<SettingsStore>>,
    pub tmdb: Arc<TmdbClient>,
}
