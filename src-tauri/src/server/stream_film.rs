use std::{error::Error, sync::Arc};

use axum::{
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use rusqlite::params;
use tokio::task;

use axum::{body::Bytes, http::HeaderValue};
use std::io::SeekFrom;
use tokio::sync::Mutex;

use tokio::fs::File as TokioFile; // Import Tokio's file type for async file operations
use tokio::io::{AsyncReadExt, AsyncSeekExt}; // For async reading and seeking

use crate::database::create_connection;

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
) -> Result<impl IntoResponse, StatusCode> {
    // Build the full path and open the file with Tokio
    let file_path = get_file_path(film_id)?;
    let file = TokioFile::open(&file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let file = Arc::new(Mutex::new(file)); // Use Arc to share ownership across async tasks

    // Get file size
    let file_length = file
        .lock()
        .await
        .metadata()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .len();

    // Parse the range header
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
        _ => (0, file_length - 1),
    };

    // Set headers for partial content response
    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("video/mp4")); // Change MIME type if needed
    response_headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&(end - start + 1).to_string()).unwrap(),
    );
    response_headers.insert(
        header::CONTENT_RANGE,
        HeaderValue::from_str(&format!("bytes {}-{}/{}", start, end, file_length)).unwrap(),
    );
    response_headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));

    // Read the requested byte range from the file
    let mut file = file.lock().await; // Await the lock
    file.seek(SeekFrom::Start(start))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut buffer = vec![0; (end - start + 1) as usize];
    file.read_exact(&mut buffer)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let headers = HeaderMap::from_iter([
        (header::CONTENT_TYPE, HeaderValue::from_static("video/mp4")),
        (
            header::CONTENT_LENGTH,
            HeaderValue::from(end - start + 1), // Create owned HeaderValue
        ),
        (
            header::CONTENT_RANGE,
            HeaderValue::from_str(format!("bytes {}-{}/{}", start, end, file_length).as_str())
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        ),
        (header::ACCEPT_RANGES, HeaderValue::from_static("bytes")),
    ]);

    Ok((StatusCode::PARTIAL_CONTENT, headers, Bytes::from(buffer)))
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
