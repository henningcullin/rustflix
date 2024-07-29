use rusqlite::Error as rusqliteError;
use serde_json::Error as serdeError;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(rusqliteError),
    JsonError(serdeError),
}

impl From<rusqliteError> for AppError {
    fn from(error: rusqliteError) -> Self {
        Self::DatabaseError(error)
    }
}

impl From<serdeError> for AppError {
    fn from(error: serdeError) -> Self {
        Self::JsonError(error)
    }
}
