//! Direct SQL helpers for `metadata_jobs`.

use sqlx::SqlitePool;

use crate::error::AppResult;
use crate::metadata::dispatch::ParkReason;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MetadataJob {
    pub kind: String,
    pub media_id: i64,
    #[allow(dead_code)]
    pub attempts: i64,
    #[allow(dead_code)]
    pub last_error: Option<String>,
    pub next_attempt_at: i64,
}

pub async fn enqueue(pool: &SqlitePool, kind: &str, media_id: i64) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO metadata_jobs (kind, media_id) VALUES (?1, ?2)
         ON CONFLICT(kind, media_id) DO NOTHING",
    )
    .bind(kind)
    .bind(media_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Force-enqueue resets an existing job (or inserts a new one) so the
/// worker treats it as fresh. Used by "Refresh metadata" and "Unlink".
pub async fn force_enqueue(pool: &SqlitePool, kind: &str, media_id: i64) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO metadata_jobs (kind, media_id) VALUES (?1, ?2)
         ON CONFLICT(kind, media_id) DO UPDATE SET
             attempts = 0,
             next_attempt_at = strftime('%s','now'),
             last_error = NULL",
    )
    .bind(kind)
    .bind(media_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Returns the next job whose next_attempt_at <= now and that isn't parked
/// on either sentinel. None ⇒ queue is empty / fully parked.
pub async fn next_due(pool: &SqlitePool) -> AppResult<Option<MetadataJob>> {
    let job: Option<MetadataJob> = sqlx::query_as(
        "SELECT kind, media_id, attempts, last_error, next_attempt_at
         FROM metadata_jobs
         WHERE COALESCE(last_error, '') NOT IN ('tmdb_auth_required', 'no_provider_available')
           AND attempts < 8
           AND next_attempt_at <= strftime('%s','now')
         ORDER BY next_attempt_at ASC
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    Ok(job)
}

pub async fn park_with_reason(
    pool: &SqlitePool,
    kind: &str,
    media_id: i64,
    reason: ParkReason,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE metadata_jobs SET last_error = ?3
         WHERE kind = ?1 AND media_id = ?2",
    )
    .bind(kind)
    .bind(media_id)
    .bind(reason.sentinel())
    .execute(pool)
    .await?;

    Ok(())
}

/// Clears the park sentinels from every parked row so the worker can pick
/// them up. Called when the user saves a new TMDB key or changes settings
/// that may unblock previously-stuck jobs.
pub async fn wake_parked(pool: &SqlitePool) -> AppResult<()> {
    sqlx::query(
        "UPDATE metadata_jobs SET
             last_error = NULL,
             next_attempt_at = strftime('%s','now')
         WHERE last_error IN ('tmdb_auth_required', 'no_provider_available')",
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Exponential backoff: attempts++; next_attempt_at = now + min(60·2^attempts, 3600).
pub async fn record_failure(
    pool: &SqlitePool,
    kind: &str,
    media_id: i64,
    message: &str,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE metadata_jobs SET
             attempts = attempts + 1,
             next_attempt_at = strftime('%s','now') +
                 CASE WHEN (60 * (1 << MIN(attempts + 1, 6))) > 3600
                      THEN 3600
                      ELSE 60 * (1 << MIN(attempts + 1, 6))
                 END,
             last_error = ?3
         WHERE kind = ?1 AND media_id = ?2",
    )
    .bind(kind)
    .bind(media_id)
    .bind(message)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_in_tx(
    conn: &mut sqlx::SqliteConnection,
    kind: &str,
    media_id: i64,
) -> AppResult<()> {
    sqlx::query("DELETE FROM metadata_jobs WHERE kind = ?1 AND media_id = ?2")
        .bind(kind)
        .bind(media_id)
        .execute(conn)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn fresh_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("memory pool");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrations");
        pool
    }

    async fn seed_show(pool: &SqlitePool) -> i64 {
        sqlx::query("INSERT INTO libraries (id, path, kind) VALUES (1, '/tmp', 'series')")
            .execute(pool)
            .await
            .expect("library");
        sqlx::query(
            "INSERT INTO shows (library_id, title, folder_path, fingerprint)
             VALUES (1, 'Test', '/tmp/test', 'test')",
        )
        .execute(pool)
        .await
        .expect("show");
        sqlx::query_scalar::<_, i64>("SELECT last_insert_rowid()")
            .fetch_one(pool)
            .await
            .expect("show id")
    }

    #[tokio::test]
    async fn enqueue_then_next_due_returns_the_job() {
        let pool = fresh_pool().await;
        let show_id = seed_show(&pool).await;

        enqueue(&pool, "show", show_id).await.unwrap();

        let job = next_due(&pool).await.unwrap().expect("a job");
        assert_eq!(job.kind, "show");
        assert_eq!(job.media_id, show_id);
        assert_eq!(job.attempts, 0);
    }

    #[tokio::test]
    async fn enqueue_twice_is_a_noop() {
        let pool = fresh_pool().await;
        let show_id = seed_show(&pool).await;

        enqueue(&pool, "show", show_id).await.unwrap();
        enqueue(&pool, "show", show_id).await.unwrap();

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM metadata_jobs WHERE kind = 'show' AND media_id = ?1",
        )
        .bind(show_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn park_with_tmdb_auth_excludes_job_from_next_due() {
        let pool = fresh_pool().await;
        let show_id = seed_show(&pool).await;
        enqueue(&pool, "show", show_id).await.unwrap();

        park_with_reason(&pool, "show", show_id, ParkReason::TmdbAuthRequired)
            .await
            .unwrap();

        let job = next_due(&pool).await.unwrap();
        assert!(job.is_none(), "parked job should be excluded");
    }

    #[tokio::test]
    async fn park_with_no_provider_excludes_job_from_next_due() {
        let pool = fresh_pool().await;
        let show_id = seed_show(&pool).await;
        enqueue(&pool, "show", show_id).await.unwrap();

        park_with_reason(&pool, "show", show_id, ParkReason::NoProviderAvailable)
            .await
            .unwrap();

        let job = next_due(&pool).await.unwrap();
        assert!(job.is_none(), "parked job should be excluded");
    }

    #[tokio::test]
    async fn wake_parked_clears_either_sentinel() {
        let pool = fresh_pool().await;
        let library_id: i64 = 1;

        // Seed a show and a movie so we have two distinct (kind, media_id) rows.
        sqlx::query("INSERT INTO libraries (id, path, kind) VALUES (?1, '/tmp', 'mixed')")
            .bind(library_id)
            .execute(&pool)
            .await
            .expect("library");
        sqlx::query(
            "INSERT INTO shows (library_id, title, folder_path, fingerprint)
             VALUES (?1, 'TestShow', '/tmp/show', 'testshow')",
        )
        .bind(library_id)
        .execute(&pool)
        .await
        .expect("show");
        let show_id: i64 = sqlx::query_scalar("SELECT last_insert_rowid()")
            .fetch_one(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO movies (library_id, title, path)
             VALUES (?1, 'TestMovie', '/tmp/movie.mkv')",
        )
        .bind(library_id)
        .execute(&pool)
        .await
        .expect("movie");
        let movie_id: i64 = sqlx::query_scalar("SELECT last_insert_rowid()")
            .fetch_one(&pool)
            .await
            .unwrap();

        enqueue(&pool, "show", show_id).await.unwrap();
        enqueue(&pool, "movie", movie_id).await.unwrap();

        park_with_reason(&pool, "show", show_id, ParkReason::TmdbAuthRequired)
            .await
            .unwrap();
        park_with_reason(&pool, "movie", movie_id, ParkReason::NoProviderAvailable)
            .await
            .unwrap();

        // Both rows should be excluded from next_due before wake.
        assert!(next_due(&pool).await.unwrap().is_none());

        wake_parked(&pool).await.unwrap();

        // After wake, both rows should be available; pull one then the other.
        let first = next_due(&pool).await.unwrap().expect("first job back");
        assert!(first.last_error.is_none());

        // Drain it so the second can surface.
        let mut conn = pool.acquire().await.unwrap();
        delete_in_tx(&mut *conn, &first.kind, first.media_id)
            .await
            .unwrap();
        drop(conn);

        let second = next_due(&pool).await.unwrap().expect("second job back");
        assert!(second.last_error.is_none());
        assert_ne!((first.kind, first.media_id), (second.kind, second.media_id));
    }

    #[tokio::test]
    async fn record_failure_increments_attempts_and_backs_off() {
        let pool = fresh_pool().await;
        let show_id = seed_show(&pool).await;
        enqueue(&pool, "show", show_id).await.unwrap();

        record_failure(&pool, "show", show_id, "boom").await.unwrap();

        let (attempts, next_attempt_at, last_error): (i64, i64, Option<String>) = sqlx::query_as(
            "SELECT attempts, next_attempt_at, last_error FROM metadata_jobs
             WHERE kind = 'show' AND media_id = ?1",
        )
        .bind(show_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(attempts, 1);
        assert_eq!(last_error.as_deref(), Some("boom"));

        let now: i64 = sqlx::query_scalar("SELECT CAST(strftime('%s','now') AS INTEGER)")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(next_attempt_at >= now + 60 && next_attempt_at <= now + 200);
    }

    #[tokio::test]
    async fn force_enqueue_resets_existing_job() {
        let pool = fresh_pool().await;
        let show_id = seed_show(&pool).await;
        enqueue(&pool, "show", show_id).await.unwrap();
        record_failure(&pool, "show", show_id, "boom").await.unwrap();
        record_failure(&pool, "show", show_id, "boom").await.unwrap();

        force_enqueue(&pool, "show", show_id).await.unwrap();

        let job = next_due(&pool).await.unwrap().expect("a job");
        assert_eq!(job.attempts, 0);
        assert!(job.last_error.is_none());
    }
}
