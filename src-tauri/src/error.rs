use rusqlite::Error as rusqliteError;
use serde_json::Error as serdeError;
use std::fmt;

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

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AppError::DatabaseError(ref err) => write!(f, "Database error: {}", err),
            AppError::JsonError(ref err) => write!(f, "JSON error: {}", err),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            AppError::DatabaseError(ref err) => Some(err),
            AppError::JsonError(ref err) => Some(err),
        }
    }
}
