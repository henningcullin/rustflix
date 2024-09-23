use rusqlite::params;

use crate::{database::create_connection, error::AppError};

pub fn delete_character(film_id: i32, actor: i32) -> Result<usize, AppError> {
    // Establish the database connection
    let conn = create_connection()?;

    // Prepare and execute the DELETE query
    let rows_affected = conn.execute(
        "DELETE FROM characters WHERE film_id = ?1 AND actor = ?2",
        params![film_id, actor],
    )?;

    Ok(rows_affected)
}

pub fn update_character(
    film_id: i32,
    actor: i32,
    new_description: &String,
    new_actor: i32,
) -> Result<usize, AppError> {
    let conn = create_connection()?;

    let rows_affected = conn.execute(
        "UPDATE characters SET description = ?1, actor = ?2 WHERE film_id = ?3 AND actor = ?4",
        params![new_description, new_actor, film_id, actor],
    )?;

    Ok(rows_affected)
}
