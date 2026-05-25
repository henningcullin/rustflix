use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};

use sqlx::SqlitePool;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::error::{AppError, AppResult};
use crate::queries;

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(serde::Serialize, Clone)]
pub struct PlayResult {
    pub session_id: u64,
}

/// Locate the mpv sidecar that Tauri places next to our executable.
///
/// Tauri's bundler strips the target-triple suffix from `externalBin` files
/// at copy time, so the runtime name is the bare binary name — `mpv.exe` on
/// Windows, `mpv` elsewhere — regardless of dev or release.
fn mpv_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let name = if cfg!(windows) { "mpv.exe" } else { "mpv" };
    let sidecar = dir.join(name);
    if sidecar.exists() {
        Some(sidecar)
    } else {
        None
    }
}

pub async fn check_mpv() -> AppResult<()> {
    let path = mpv_path().ok_or(AppError::MpvMissing)?;
    let ok = Command::new(&path)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false);
    if ok {
        Ok(())
    } else {
        Err(AppError::MpvMissing)
    }
}

/// Spawn mpv, await its exit, then persist the final playback position.
///
/// We use mpv's `--term-status-msg` to emit `POS=<seconds> DUR=<seconds>` lines
/// that we tail on stdout. The final values are what get written back to the DB.
pub async fn play(
    pool: &SqlitePool,
    kind: &'static str,
    media_id: i64,
    path: &str,
    resume_seconds: i64,
) -> AppResult<PlayResult> {
    let session_id = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mpv = mpv_path().ok_or(AppError::MpvMissing)?;

    let mut cmd = Command::new(&mpv);
    cmd.arg(path)
        .arg("--force-window=yes")
        .arg("--keep-open=no")
        .arg("--really-quiet")
        .arg("--term-status-msg=POS=${=time-pos} DUR=${=duration}");

    if resume_seconds > 5 {
        cmd.arg(format!("--start=+{}", resume_seconds));
    }

    cmd.stdout(Stdio::piped()).stderr(Stdio::null());

    let mut child = cmd.spawn().map_err(|_| AppError::MpvMissing)?;
    let stdout = child.stdout.take();

    let parse_task = tokio::spawn(async move {
        let mut last_pos: i64 = 0;
        let mut last_dur: Option<i64> = None;
        if let Some(stdout) = stdout {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if let Some(rest) = line.strip_prefix("POS=") {
                    let mut parts = rest.splitn(2, " DUR=");
                    if let Some(pos) = parts.next() {
                        if let Ok(p) = pos.parse::<f64>() {
                            last_pos = p as i64;
                        }
                    }
                    if let Some(dur) = parts.next() {
                        if let Ok(d) = dur.parse::<f64>() {
                            last_dur = Some(d as i64);
                        }
                    }
                }
            }
        }
        (last_pos, last_dur)
    });

    // Wait for both halves regardless of which errors first — losing the
    // child's exit error is worse than losing progress, so we drain the
    // parse task no matter what and save whatever it captured.
    let wait_result = child.wait().await;
    let parse_result = parse_task.await;

    let (final_pos, final_dur) = match parse_result {
        Ok(values) => values,
        Err(join_error) => {
            eprintln!("mpv parse task failed: {join_error}");
            (0, None)
        }
    };

    let watched = match final_dur {
        Some(duration) if duration > 0 => {
            (final_pos as f64 / duration as f64) >= 0.9 || (duration - final_pos) <= 60
        }
        _ => false,
    };

    queries::upsert_progress(pool, kind, media_id, final_pos, final_dur, watched).await?;

    // Surface the child wait error after we've already persisted progress,
    // so a failed reap doesn't drop the last-known position.
    wait_result?;

    Ok(PlayResult { session_id })
}
