use serde::{Deserialize, Serialize};

use crate::FromRow;

#[derive(Serialize, Deserialize, Debug)]
pub struct Genre {
    pub id: u32,
    pub name: String,
}

impl FromRow for Genre {
    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        Ok(Genre {
            id: row.get("id")?,
            name: row.get("name")?,
        })
    }
}
