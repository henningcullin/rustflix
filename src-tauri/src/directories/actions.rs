use rusqlite::params;

use crate::database::create_connection;

pub fn add_directory(path: &str) -> Result<(), rusqlite::Error> {
    let conn = create_connection()?;

    conn.execute("INSERT INTO directories (path) VALUES (?1)", params![path])?;

    Ok(())
}
