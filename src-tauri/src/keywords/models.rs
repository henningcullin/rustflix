use serde::Serialize;

use crate::FromRow;

#[derive(Debug, Serialize)]
pub struct Keyword {
    pub id: u32,
    pub name: String,
}

impl FromRow for Keyword {
    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        Ok(Keyword {
            id: row.get("id")?,
            name: row.get("name")?,
        })
    }
}
