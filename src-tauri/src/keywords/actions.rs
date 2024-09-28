use rusqlite::params;

use crate::{database::create_connection, error::AppError};

pub fn create_keyword(film_id: i32, keyword: &String) -> Result<usize, AppError> {
    let conn = create_connection()?;

    let rows_affected = conn.execute(
        "INSERT INTO film_keywords (film_id, keyword) VALUES (?1, ?2)",
        params![film_id, keyword],
    )?;

    Ok(rows_affected)
}
