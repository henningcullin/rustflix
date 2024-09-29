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
