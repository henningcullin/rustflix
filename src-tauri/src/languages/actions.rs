use rusqlite::params;

use crate::{database::create_connection, error::AppError, FromRow};

use super::Language;

pub fn get_all_languages() -> Result<Vec<Language>, AppError> {
    let conn = create_connection()?;

    let mut stmt = conn.prepare("SELECT id, name FROM languages")?;

    let languages: Vec<Language> = stmt
        .query_map([], Language::from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(languages)
}

pub fn add_language_to_film(film_id: i32, language_id: &i32) -> Result<usize, AppError> {
    let conn = create_connection()?;

    let rows_affected = conn.execute(
        "INSERT INTO film_languages (film_id, language_id) VALUES (?1, ?2)",
        params![film_id, language_id],
    )?;

    Ok(rows_affected)
}

pub fn remove_language_from_film(film_id: i32, language_id: &i32) -> Result<usize, AppError> {
    let conn = create_connection()?;

    let rows_affected = conn.execute(
        "DELETE FROM film_languages WHERE film_id = ?1 AND language_id = ?2",
        params![film_id, language_id],
    )?;

    Ok(rows_affected)
}
