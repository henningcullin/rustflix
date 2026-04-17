mod db;
mod directories;
mod error;
mod films;
mod images;
mod settings;
mod state;
mod stream;
mod tmdb;

use std::sync::Arc;

use tauri::Manager;
use tokio::sync::RwLock;

use crate::settings::SettingsStore;
use crate::state::AppState;
use crate::tmdb::TmdbClient;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .register_asynchronous_uri_scheme_protocol("stream", stream::handle)
        .setup(|app| {
            let handle = app.handle().clone();
            let app_data_dir = handle
                .path()
                .app_data_dir()
                .expect("app data dir resolves");
            std::fs::create_dir_all(&app_data_dir).ok();

            let db_path = app_data_dir.join("rustflix.sqlite");
            let store = SettingsStore::load(&app_data_dir).expect("settings load");
            let initial_key = store.tmdb_key();
            let settings = Arc::new(RwLock::new(store));
            let tmdb = TmdbClient::new(initial_key);

            let pool = tauri::async_runtime::block_on(async { db::init(&db_path).await })
                .expect("db init");

            app.manage(AppState {
                db: pool,
                app_data_dir,
                settings,
                tmdb,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            settings::get_settings,
            settings::set_tmdb_api_key,
            directories::add_directory,
            directories::list_directories,
            directories::delete_directory,
            directories::scan_directory,
            films::list_films,
            films::get_film,
            films::delete_film,
            films::set_left_off_point,
            tmdb::tmdb_search,
            tmdb::tmdb_import_film,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
