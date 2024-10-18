use rusqlite::params;

use crate::{database::create_connection, error::AppError, FromRow};

use super::Keyword;

pub fn get_all_keywords() -> Result<Vec<Keyword>, AppError> {
    let conn = create_connection()?;

    let mut stmt = conn.prepare("SELECT id, name FROM keywords")?;

    let keywords: Vec<Keyword> = stmt
        .query_map([], Keyword::from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(keywords)
}

pub fn create_keyword(film_id: i32, keyword_id: i32) -> Result<usize, AppError> {
    let conn = create_connection()?;

    let rows_affected = conn.execute(
        "INSERT INTO film_keywords (film_id, keyword_id) VALUES (?1, ?2)",
        params![film_id, keyword_id],
    )?;

    Ok(rows_affected)
}

pub fn delete_keyword(film_id: i32, keyword_id: i32) -> Result<usize, AppError> {
    let conn = create_connection()?;

    let rows_affected = conn.execute(
        "DELETE FROM film_keywords WHERE film_id = ?1 AND keyword_id = ?2",
        params![film_id, keyword_id],
    )?;

    Ok(rows_affected)
}

pub fn add_keyword_to_film(film_id: i32, keyword_id: &i32) -> Result<usize, AppError> {
    let conn = create_connection()?;

    let rows_affected = conn.execute(
        "INSERT INTO film_keywords (film_id, keyword_id) VALUES (?1, ?2)",
        params![film_id, keyword_id],
    )?;

    Ok(rows_affected)
}

pub fn remove_keyword_from_film(film_id: i32, keyword_id: &i32) -> Result<usize, AppError> {
    let conn = create_connection()?;

    let rows_affected = conn.execute(
        "DELETE FROM film_keywords WHERE film_id = ?1 AND keyword_id = ?2",
        params![film_id, keyword_id],
    )?;

    Ok(rows_affected)
}
