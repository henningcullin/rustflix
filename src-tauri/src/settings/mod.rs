use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::state::AppState;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub tmdb_api_key: Option<String>,
}

pub struct SettingsStore {
    path: PathBuf,
    data: Settings,
}

impl SettingsStore {
    pub fn load(app_data_dir: &Path) -> AppResult<Self> {
        let path = app_data_dir.join("settings.json");
        let data = if path.exists() {
            let raw = std::fs::read_to_string(&path)?;
            serde_json::from_str(&raw)?
        } else {
            Settings::default()
        };
        Ok(Self { path, data })
    }

    pub fn get(&self) -> Settings {
        self.data.clone()
    }

    pub fn update(&mut self, settings: Settings) -> AppResult<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let raw = serde_json::to_string_pretty(&settings)?;
        std::fs::write(&self.path, raw)?;
        self.data = settings;
        Ok(())
    }

    pub fn tmdb_key(&self) -> Option<String> {
        self.data.tmdb_api_key.clone()
    }
}

#[tauri::command]
pub async fn get_settings(state: tauri::State<'_, AppState>) -> AppResult<Settings> {
    Ok(state.settings.read().await.get())
}

#[tauri::command]
pub async fn set_tmdb_api_key(
    state: tauri::State<'_, AppState>,
    key: String,
) -> AppResult<Settings> {
    let trimmed = key.trim().to_string();
    let key = if trimmed.is_empty() { None } else { Some(trimmed) };

    let mut store = state.settings.write().await;
    let mut current = store.get();
    current.tmdb_api_key = key.clone();
    store.update(current.clone())?;
    drop(store);

    state.tmdb.set_api_key(key).await;
    Ok(current)
}

