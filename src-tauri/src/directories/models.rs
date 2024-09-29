use serde::Serialize;

use crate::FromRow;

#[derive(Debug, Serialize)]
pub struct Directory {
    pub id: u32,
    pub path: String,
}

impl FromRow for Directory {
    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        Ok(Directory {
            id: row.get("id")?,
            path: row.get("path")?,
        })
    }
}
