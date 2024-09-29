use serde::Serialize;

use crate::FromRow;

#[derive(Debug, Serialize)]
pub struct Language {
    pub id: u32,
    pub name: String,
}

impl FromRow for Language {
    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        Ok(Language {
            id: row.get("id")?,
            name: row.get("name")?,
        })
    }
}
