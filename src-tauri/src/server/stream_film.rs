use std::{error::Error, sync::Arc};

use axum::{
    body::StreamBody,
    extract::Path,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use rusqlite::params;
use tokio::{fs::File, task};
use tokio_util::io::ReaderStream;

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

async fn stream_film(Path(film_id): Path<i32>) -> Result<impl IntoResponse, StatusCode> {
    // Get the file path from the database (this function must return a Result<String, StatusCode>)
    let file_path = get_file_path(film_id)?;

    // Open the file asynchronously
    let file = File::open(&file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Create a stream from the file
    let stream = ReaderStream::new(file);

    // Wrap the stream in a StreamBody
    let body = StreamBody::new(stream);

    // Return a response with appropriate headers
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "video/mp4"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        body,
    ))
}

// Function to create the axum router
fn create_router() -> Router {
    Router::new().route("/film/:id", get(stream_film))
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
