use reqwest::Error as reqwestError;
use rusqlite::Error as rusqliteError;
use scraper::error::SelectorErrorKind as SelectorError;
use serde_json::Error as serdeError;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(rusqlite::Error),
    JsonError(serdeError),
    SelectorError(SelectorError),
    ReqwestError(reqwestError),
    ScrapeError(String),
}

impl From<rusqliteError> for AppError {
    fn from(value: rusqliteError) -> Self {
        Self::DatabaseError(value)
    }
}

impl From<serdeError> for AppError {
    fn from(value: serdeError) -> Self {
        Self::JsonError(value)
    }
}

impl From<SelectorError> for AppError {
    fn from(value: SelectorError) -> Self {
        Self::SelectorError(value)
    }
}

impl From<reqwestError> for AppError {
    fn from(value: reqwestError) -> Self {
        Self::ReqwestError(value)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AppError::DatabaseError(ref err) => write!(f, "Database error: {}", err),
            AppError::JsonError(ref err) => write!(f, "JSON error: {}", err),
            AppError::SelectorError(ref err) => write!(f, "Selector error: {}", err),
            AppError::ScrapeError(ref err) => write!(f, "Scrape error: {}", err),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            AppError::DatabaseError(ref err) => Some(err),
            AppError::JsonError(ref err) => Some(err),
            AppError::SelectorError(ref err) => Some(err),
            AppError::ScrapeError(_) => None,
        }
    }
}
