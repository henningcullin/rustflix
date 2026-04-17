use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::state::AppState;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub tmdb_api_key: Option<String>,
    #[serde(default)]
    pub alias: Option<String>,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub first_run_completed: bool,
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

async fn patch<F: FnOnce(&mut Settings)>(
    state: &tauri::State<'_, AppState>,
    f: F,
) -> AppResult<Settings> {
    let mut store = state.settings.write().await;
    let mut current = store.get();
    f(&mut current);
    store.update(current.clone())?;
    Ok(current)
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

    let next = patch(&state, |s| s.tmdb_api_key = key.clone()).await?;
    state.tmdb.set_api_key(key).await;
    Ok(next)
}

#[tauri::command]
pub async fn set_alias(
    state: tauri::State<'_, AppState>,
    alias: String,
) -> AppResult<Settings> {
    let trimmed = alias.trim().to_string();
    let value = if trimmed.is_empty() { None } else { Some(trimmed) };
    patch(&state, |s| s.alias = value).await
}

#[tauri::command]
pub async fn set_theme(
    state: tauri::State<'_, AppState>,
    theme: String,
) -> AppResult<Settings> {
    let normalized = match theme.as_str() {
        "light" | "dark" | "system" => Some(theme),
        _ => Some("system".to_string()),
    };
    patch(&state, |s| s.theme = normalized).await
}

#[tauri::command]
pub async fn complete_first_run(state: tauri::State<'_, AppState>) -> AppResult<Settings> {
    patch(&state, |s| s.first_run_completed = true).await
}

#[tauri::command]
pub async fn reset_first_run(state: tauri::State<'_, AppState>) -> AppResult<Settings> {
    patch(&state, |s| s.first_run_completed = false).await
}
