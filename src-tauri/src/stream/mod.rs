use std::path::PathBuf;

use sqlx::SqlitePool;
use tauri::http::{header, Request, Response, StatusCode};
use tauri::{Manager, UriSchemeContext};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};

use crate::state::AppState;

const DEFAULT_CHUNK: u64 = 1_048_576; // 1 MiB cap per request when range is open-ended.

pub fn handle<R: tauri::Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
    responder: tauri::UriSchemeResponder,
) {
    let app = ctx.app_handle().clone();
    let state = app.state::<AppState>();
    let db = state.db.clone();

    tauri::async_runtime::spawn(async move {
        let resp = match build_response(&request, &db).await {
            Ok(resp) => resp,
            Err(err) => error_response(err),
        };
        responder.respond(resp);
    });
}

async fn build_response(
    request: &Request<Vec<u8>>,
    db: &SqlitePool,
) -> Result<Response<Vec<u8>>, StreamError> {
    if request.method() != tauri::http::Method::GET {
        return Err(StreamError::MethodNotAllowed);
    }

    let uri = request.uri();
    // stream://film/{id} — under WebView2 it arrives as http://stream.localhost/film/{id}
    // Parse by path segments.
    let path = uri.path();
    let mut segments = path.trim_start_matches('/').split('/');
    let kind = segments.next().unwrap_or("");
    let id_raw = segments.next().unwrap_or("");
    if kind != "film" || id_raw.is_empty() {
        return Err(StreamError::NotFound);
    }
    let film_id: i64 = id_raw.parse().map_err(|_| StreamError::NotFound)?;

    let file_path: Option<String> =
        sqlx::query_scalar("SELECT file_path FROM films WHERE id = ?")
            .bind(film_id)
            .fetch_optional(db)
            .await
            .map_err(|e| StreamError::Internal(e.to_string()))?;
    let file_path = PathBuf::from(file_path.ok_or(StreamError::NotFound)?);

    let metadata = tokio::fs::metadata(&file_path)
        .await
        .map_err(|e| StreamError::Internal(format!("stat: {e}")))?;
    let total = metadata.len();

    let mime = mime_guess::from_path(&file_path)
        .first_or_octet_stream()
        .to_string();

    let range_header = request
        .headers()
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned());

    let (start, end, status) = if let Some(range) = range_header.as_deref() {
        let (s, e) = parse_single_range(range, total)?;
        (s, e, StatusCode::PARTIAL_CONTENT)
    } else {
        let end = total.saturating_sub(1).min(DEFAULT_CHUNK.saturating_sub(1));
        (0, end, StatusCode::OK)
    };

    if end >= total || start > end {
        return Err(StreamError::InvalidRange(total));
    }

    let length = end - start + 1;
    let mut file = File::open(&file_path)
        .await
        .map_err(|e| StreamError::Internal(format!("open: {e}")))?;
    file.seek(SeekFrom::Start(start))
        .await
        .map_err(|e| StreamError::Internal(format!("seek: {e}")))?;
    let mut buf = vec![0u8; length as usize];
    file.read_exact(&mut buf)
        .await
        .map_err(|e| StreamError::Internal(format!("read: {e}")))?;

    let mut resp = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, mime)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_LENGTH, length.to_string())
        .header(header::CACHE_CONTROL, "no-store");

    if matches!(status, StatusCode::PARTIAL_CONTENT) || range_header.is_some() {
        resp = resp.header(
            header::CONTENT_RANGE,
            format!("bytes {start}-{end}/{total}"),
        );
    }

    resp.body(buf)
        .map_err(|e| StreamError::Internal(format!("body: {e}")))
}

fn parse_single_range(header: &str, total: u64) -> Result<(u64, u64), StreamError> {
    let range = header
        .strip_prefix("bytes=")
        .ok_or_else(|| StreamError::InvalidRange(total))?;
    let first = range.split(',').next().unwrap_or("");
    let (start_str, end_str) = first
        .split_once('-')
        .ok_or(StreamError::InvalidRange(total))?;

    let start_str = start_str.trim();
    let end_str = end_str.trim();

    let (start, end) = match (start_str.is_empty(), end_str.is_empty()) {
        (false, false) => {
            let s: u64 = start_str.parse().map_err(|_| StreamError::InvalidRange(total))?;
            let e: u64 = end_str.parse().map_err(|_| StreamError::InvalidRange(total))?;
            (s, e)
        }
        (false, true) => {
            let s: u64 = start_str.parse().map_err(|_| StreamError::InvalidRange(total))?;
            let cap_end = s + DEFAULT_CHUNK.saturating_sub(1);
            let e = cap_end.min(total.saturating_sub(1));
            (s, e)
        }
        (true, false) => {
            let suffix: u64 = end_str.parse().map_err(|_| StreamError::InvalidRange(total))?;
            if suffix == 0 {
                return Err(StreamError::InvalidRange(total));
            }
            let s = total.saturating_sub(suffix);
            (s, total.saturating_sub(1))
        }
        (true, true) => return Err(StreamError::InvalidRange(total)),
    };

    if end >= total || start > end {
        return Err(StreamError::InvalidRange(total));
    }

    // Cap the end so we never buffer an arbitrarily large range into memory.
    let capped_end = end.min(start + DEFAULT_CHUNK.saturating_sub(1));
    Ok((start, capped_end))
}

#[derive(Debug)]
enum StreamError {
    NotFound,
    MethodNotAllowed,
    InvalidRange(u64),
    Internal(String),
}

fn error_response(err: StreamError) -> Response<Vec<u8>> {
    let (status, body) = match err {
        StreamError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
        StreamError::MethodNotAllowed => {
            (StatusCode::METHOD_NOT_ALLOWED, "method not allowed".to_string())
        }
        StreamError::InvalidRange(total) => {
            return Response::builder()
                .status(StatusCode::RANGE_NOT_SATISFIABLE)
                .header(header::CONTENT_RANGE, format!("bytes */{total}"))
                .body(Vec::new())
                .expect("valid response");
        }
        StreamError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
    };
    let _ = &body;
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(body.into_bytes())
        .expect("valid response")
}
