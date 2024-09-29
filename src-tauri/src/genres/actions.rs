use crate::{database::create_connection, error::AppError, FromRow};

use super::Genre;

pub fn get_all_genres() -> Result<Vec<Genre>, AppError> {
    let conn = create_connection()?;

    let mut stmt = conn.prepare("SELECT id, name FROM genres")?;

    let genres: Vec<Genre> = stmt
        .query_map([], Genre::from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(genres)
}
