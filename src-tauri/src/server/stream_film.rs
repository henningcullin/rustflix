use std::{error::Error, sync::Arc};

use axum::{
    body::StreamBody,
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use rusqlite::params;
use tokio::task;

use std::io::SeekFrom;
use tokio_util::io::ReaderStream;

use tokio::fs::File as TokioFile; // Import Tokio's file type for async file operations
use tokio::io::{AsyncReadExt, AsyncSeekExt}; // For async reading and seeking

use crate::database::create_connection;

const INITIAL_CHUNK_SIZE: u64 = 1_048_576; // 1 MB for faster initial load

// Query to get the file path for the given film ID
fn get_file_path(film_id: i32) -> Result<String, StatusCode> {
    let conn = create_connection().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut stmt = conn
        .prepare("SELECT file FROM films WHERE id = ?")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result: Result<String, _> = stmt.query_row(params![film_id], |row| row.get(0));
    result.map_err(|_| StatusCode::NOT_FOUND)
}

async fn stream_film(
    Path(film_id): Path<i32>,
    headers: HeaderMap,
) -> Result<Response<StreamBody<ReaderStream<tokio::io::Take<tokio::fs::File>>>>, StatusCode> {
    let file_path = get_file_path(film_id)?;
    let mut file = TokioFile::open(&file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let file_length = file
        .metadata()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .len();

    let range = headers
        .get(header::RANGE)
        .and_then(|range_header| range_header.to_str().ok());
    let (start, end) = match range {
        Some(range) if range.starts_with("bytes=") => {
            let parts: Vec<&str> = range["bytes=".len()..].split('-').collect();
            let start = parts[0].parse::<u64>().unwrap_or(0);
            let end = parts
                .get(1)
                .and_then(|&s| s.parse::<u64>().ok())
                .unwrap_or(file_length - 1);
            (start, end)
        }
        _ => (0, INITIAL_CHUNK_SIZE.min(file_length) - 1),
    };

    if start >= file_length || end >= file_length || start > end {
        return Err(StatusCode::RANGE_NOT_SATISFIABLE);
    }

    // Seek to the start position for the requested range
    file.seek(SeekFrom::Start(start))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create a stream that reads from the file in chunks
    let stream = ReaderStream::new(file.take(end - start + 1));
    let body = StreamBody::new(stream);

    // Build the response with individual headers and the streaming body
    let response = Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CONTENT_LENGTH, (end - start + 1).to_string())
        .header(
            header::CONTENT_RANGE,
            format!("bytes {}-{}/{}", start, end, file_length),
        )
        .header(header::ACCEPT_RANGES, "bytes")
        .body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

// Function to create the axum router
fn create_router() -> Router {
    Router::new().route("/film/:film_id", get(stream_film))
}

pub async fn start_server() -> Result<(), Arc<dyn Error + Send + Sync>> {
    let router = create_router(); // Assuming `create_router()` returns a Router

    // Spawn the server in a separate task to run independently
    task::spawn(async move {
        if let Err(e) = axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
            .serve(router.into_make_service())
            .await
        {
            eprintln!("Server error: {}", e); // Log the error to stderr
        }
    });

    Ok(())
}
