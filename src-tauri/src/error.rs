use rusqlite::Error as rusqliteError;
use serde_json::Error as serdeError;

pub enum Error {
    DatabaseError(rusqliteError),
    JsonError(serdeError),
}

impl From<rusqliteError> for Error {
    fn from(error: rusqliteError) -> Self {
        Self::DatabaseError(error)
    }
}

impl From<serdeError> for Error {
    fn from(error: serdeError) -> Self {
        Self::JsonError(error)
    }
}
