use rusqlite::params;

use crate::{
    database::{create_connection, delete, insert},
    error::AppError,
    FromRow,
};

use super::Genre;

pub fn get_all_genres() -> Result<Vec<Genre>, AppError> {
    let conn = create_connection()?;

    let mut stmt = conn.prepare("SELECT id, name FROM genres")?;

    let genres: Vec<Genre> = stmt
        .query_map([], Genre::from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(genres)
}

pub fn add_genre_to_film(film_id: i32, genre_id: &i32) -> Result<i32, AppError> {
    insert(
        &create_connection()?,
        "INSERT INTO film_genres (film_id, genre_id) VALUES (?1, ?2)",
        params![film_id, genre_id],
    )
}

pub fn remove_genre_from_film(film_id: i32, genre_id: &i32) -> Result<usize, AppError> {
    delete(
        &create_connection()?,
        "DELETE FROM film_genres WHERE film_id = ?1 AND genre_id = ?2",
        params![film_id, genre_id],
    )
}
