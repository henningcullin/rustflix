use rusqlite::params;

use crate::{
    database::{create_connection, delete, insert},
    error::AppError,
};

pub fn delete_director(film_id: i32, person_id: i32) -> Result<usize, AppError> {
    delete(
        &create_connection()?,
        "DELETE FROM film_directors WHERE film_id = ?1 AND person_id = ?2",
        params![film_id, person_id],
    )
}

pub fn create_director(film_id: i32, person_id: i32) -> Result<i32, AppError> {
    insert(
        &create_connection()?,
        "INSERT INTO film_directors (film_id, person_id) VALUES (?1, ?2)",
        params![film_id, person_id],
    )
}
