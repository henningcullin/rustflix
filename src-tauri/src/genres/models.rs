use rusqlite::types::{FromSql, FromSqlError, ValueRef};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Genre {
    pub id: u32,
    pub name: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenreList(pub Vec<Genre>);

impl FromSql for GenreList {
    fn column_result(value: ValueRef) -> Result<Self, FromSqlError> {
        let value_str = value.as_str()?;
        let genres: Vec<Genre> =
            serde_json::from_str(value_str).map_err(|_| FromSqlError::InvalidType)?;
        Ok(GenreList(genres))
    }
}
