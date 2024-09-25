// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod characters;
mod database;
mod directories;
mod directors;
mod error;
mod films;
mod genres;
mod images;
mod languages;
mod persons;
mod scrapers;

use characters::{delete_character, update_character};
use directories::{add_directory, delete_directory, get_all_directories, select_directory};
use films::{get_all_films, get_film, sync_new_films};
use images::{get_avatar, get_cover};
use persons::get_all_persons;
use scrapers::scrape_film;

#[tauri::command]
async fn fetch_data(url: String) -> Result<String, String> {
    let response = reqwest::get(url).await;
    match response {
        Ok(res) => match res.text().await {
            Ok(body) => Ok(body),
            Err(error) => Err(error.to_string()),
        },
        Err(error) => Err(error.to_string()),
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
            sync_new_films,
            // MISC
            fetch_data,
            // SCRAPER
            scrape_film,
            // IMAGES
            get_avatar,
            get_cover,
            // CHARACTERS
            delete_character,
            update_character,
            // PERSONS
            get_all_persons
        ])
        .setup(|_app| {
            match database::initialize_database() {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("{err:?}");
                }
            };
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
