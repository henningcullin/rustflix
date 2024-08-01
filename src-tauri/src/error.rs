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
    SelectorError(SelectorError<'static>),
    ReqwestError(reqwestError),
    ScrapeError(String),
}

impl AppError {
    pub fn new() -> Self {
        Self::ScrapeError("Option was none".into())
    }
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

impl From<SelectorError<'static>> for AppError {
    fn from(value: SelectorError<'static>) -> Self {
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
            Self::DatabaseError(ref err) => write!(f, "Database error: {}", err),
            Self::JsonError(ref err) => write!(f, "JSON error: {}", err),
            Self::SelectorError(ref err) => write!(f, "Selector error: {}", err),
            Self::ScrapeError(ref err) => write!(f, "Scrape error: {}", err),
            Self::ReqwestError(ref err) => write!(f, "Reqwest error: {}", err),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            Self::DatabaseError(ref err) => Some(err),
            Self::JsonError(ref err) => Some(err),
            Self::SelectorError(ref err) => Some(err),
            Self::ScrapeError(_) => None,
            Self::ReqwestError(ref err) => Some(err),
        }
    }
}
