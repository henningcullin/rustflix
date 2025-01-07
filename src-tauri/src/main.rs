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
mod keywords;
mod languages;
mod persons;
mod scrapers;
mod server;
#[macro_use]
mod macros;

use characters::{create_character, delete_character, update_character};
use directories::{
    create_directory, delete_directory, get_all_directories, select_directory, update_directory,
};
use directors::{create_director, delete_director};
use films::{delete_film, get_all_films, get_film, set_left_off_point};
use genres::{add_genre_to_film, get_all_genres, remove_genre_from_film};
use images::{get_avatar, get_cover};
use keywords::{
    add_keyword_to_film, create_keyword, delete_keyword, get_all_keywords, remove_keyword_from_film,
};
use languages::{add_language_to_film, get_all_languages, remove_language_from_film};
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

pub trait FromRow: Sized {
    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error>;
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // DIRECTORIES
            create_directory,
            update_directory,
            delete_directory,
            select_directory,
            get_all_directories,
            // FILMS
            delete_film,
            get_all_films,
            get_film,
            set_left_off_point,
            // MISC
            fetch_data,
            // SCRAPER
            scrape_film,
            // GENRES
            get_all_genres,
            add_genre_to_film,
            remove_genre_from_film,
            // LANGUAGES
            get_all_languages,
            add_language_to_film,
            remove_language_from_film,
            // KEYWORDS
            get_all_keywords,
            create_keyword,
            delete_keyword,
            add_keyword_to_film,
            remove_keyword_from_film,
            // IMAGES
            get_avatar,
            get_cover,
            // CHARACTERS
            delete_character,
            update_character,
            create_character,
            // PERSONS
            get_all_persons,
            // DIRECTORS
            delete_director,
            create_director,
        ])
        .setup(|_app| {
            match database::initialize_database() {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("{err:?}");
                }
            };

            // Start the axum server in an async task
            tauri::async_runtime::spawn(async {
                if let Err(e) = server::stream_film::start_server().await {
                    eprintln!("Failed to start axum server: {e}");
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
