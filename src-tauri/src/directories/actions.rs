use std::path::PathBuf;

use rusqlite::params;
use tauri::api::dialog::blocking::FileDialogBuilder;

use crate::{
    database::{create_connection, sql},
    error::AppError,
    FromRow,
};

use super::Directory;

pub fn add_directory(path: &str) -> Result<i32, AppError> {
    sql(
        &create_connection()?,
        "INSERT INTO directories (path) VALUES (?1) RETURNING id",
        params![path],
    )
}

pub fn remove_directory(id: u32) -> Result<(), rusqlite::Error> {
    let conn = create_connection()?;

    conn.execute("DELETE FROM directories WHERE id = ?1", params![id])?;

    Ok(())
}

pub fn get_all_directories() -> Result<Vec<Directory>, rusqlite::Error> {
    let conn = create_connection()?;

    let mut stmt = conn.prepare("SELECT id, path FROM directories")?;
    let directories: Vec<Directory> = stmt
        .query_map([], Directory::from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(directories)
}

pub async fn select_directory() -> Option<PathBuf> {
    FileDialogBuilder::new().pick_folder()
}
