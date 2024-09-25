use rusqlite::params;

use crate::{database::create_connection, error::AppError};

pub fn delete_director(film_id: i32, person_id: i32) -> Result<usize, AppError> {
    // Establish the database connection
    let conn = create_connection()?;

    // Prepare and execute the DELETE query
    let rows_affected = conn.execute(
        "DELETE FROM film_directors WHERE film_id = ?1 AND person_id = ?2",
        params![film_id, person_id],
    )?;

    Ok(rows_affected)
}
