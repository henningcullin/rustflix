// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod directories;
mod error;
mod films;

use directories::{add_directory, delete_directory, get_all_directories, select_directory};
use films::{get_all_files, get_all_films};

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
            get_all_files,
        ])
        .setup(|_app| {
            match database::initialize_database() {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("{:?}", err);
                }
            };
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
