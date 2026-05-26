mod admin;
mod commands;
mod db;
mod error;
mod metadata;
mod models;
mod player;
mod queries;
mod scanner;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            // sqlx requires async setup; block on it here once at startup.
            let pool = tauri::async_runtime::block_on(db::open(&app_data_dir))
                .expect("failed to open database");
            app.manage(pool);

            let http_client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .user_agent(concat!("rustflix/", env!("CARGO_PKG_VERSION")))
                .build()
                .expect("failed to build reqwest client");
            app.manage(http_client.clone());

            let pool_for_worker = app.state::<db::Db>().inner().clone();
            let notify =
                metadata::worker::spawn(pool_for_worker, http_client, app.handle().clone());
            app.manage(notify);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_libraries,
            commands::add_library,
            commands::remove_library,
            commands::scan_libraries,
            commands::list_movies,
            commands::get_movie,
            commands::list_shows,
            commands::get_show,
            commands::get_seasons,
            commands::get_episode,
            commands::continue_watching,
            commands::set_watched,
            commands::check_mpv,
            commands::play_movie,
            commands::play_episode,
            commands::update_show_metadata,
            commands::update_movie_metadata,
            commands::update_episode_title,
            commands::merge_shows,
            commands::delete_show,
            commands::set_show_poster_from_file,
            commands::set_movie_poster_from_file,
            commands::reset_show_poster,
            commands::reset_movie_poster,
            commands::get_app_setting,
            commands::set_app_setting,
            commands::metadata_status_counts,
            commands::refresh_metadata,
            commands::unlink_metadata,
            commands::metadata_search,
            commands::link_metadata,
            commands::list_needs_review,
            commands::admin_list_rows,
            commands::admin_update_row,
            commands::admin_delete_rows,
            commands::admin_fk_label,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
