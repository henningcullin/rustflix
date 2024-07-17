// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod directories;
mod films;

use directories::{add_directory, select_directory};
use films::get_all_films;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            add_directory,
            select_directory,
            get_all_films
        ])
        .setup(|_app| {
            match database::initialize_database() {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("{:?}", err);
                }
            };
            match database::check_for_new_films() {
                Ok(_) => {}
                Err(_) => {}
            };
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
