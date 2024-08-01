// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod directories;
mod error;
mod films;
mod scrapers;

use directories::{add_directory, delete_directory, get_all_directories, select_directory};
use films::{get_all_films, get_film};
use scrapers::scrape_film;

#[tauri::command]
async fn fetch_data(url: String) -> Result<String, String> {
    let response = reqwest::get(url).await;
    match response {
        Ok(res) => {
            let body = res.text().await.unwrap_or_default();
            Ok(body)
        }
        Err(err) => Err(err.to_string()),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // DIRECTORIES
            add_directory,
            delete_directory,
            select_directory,
            get_all_directories,
            // FILMS
            get_all_films,
            get_film,
            // MISC
            fetch_data,
            // SCRAPER
            scrape_film
        ])
        .setup(|_app| {
            match database::initialize_database() {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("{err:?}");
                }
            };
            match films::sync_films_with_files() {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("{err:?}");
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
