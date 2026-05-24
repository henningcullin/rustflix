//! Admin / registervård backend. Generic CRUD over the seven persistent
//! tables. See docs/superpowers/specs/2026-05-24-admin-area-design.md.

use std::collections::HashMap;

use serde::Deserialize;
use serde_json::{Map, Value};
use sqlx::{Column, Row, SqlitePool, TypeInfo, ValueRef};

use crate::error::{AppError, AppResult};

/// The seven tables the admin knows about. Frontend deserialises into this
/// enum or fails — that's the SQL-injection guardrail.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Table {
    Libraries,
    Shows,
    Movies,
    Episodes,
    WatchHistory,
    MetadataJobs,
    AppSettings,
}

impl Table {
    fn name(&self) -> &'static str {
        match self {
            Table::Libraries => "libraries",
            Table::Shows => "shows",
            Table::Movies => "movies",
            Table::Episodes => "episodes",
            Table::WatchHistory => "watch_history",
            Table::MetadataJobs => "metadata_jobs",
            Table::AppSettings => "app_settings",
        }
    }

    fn primary_key(&self) -> &'static [&'static str] {
        match self {
            Table::Libraries
            | Table::Shows
            | Table::Movies
            | Table::Episodes => &["id"],
            Table::WatchHistory => &["media_kind", "media_id"],
            Table::MetadataJobs => &["kind", "media_id"],
            Table::AppSettings => &["key"],
        }
    }

    fn columns(&self) -> &'static [&'static str] {
        match self {
            Table::Libraries => &["id", "path", "kind", "added_at"],
            Table::Shows => &[
                "id", "library_id", "title", "year", "folder_path", "fingerprint",
                "poster_path", "poster_origin", "overview", "added_at",
                "provider", "provider_id", "rating", "genres", "top_cast",
                "first_air_date", "metadata_synced_at", "metadata_locked",
            ],
            Table::Movies => &[
                "id", "library_id", "title", "year", "path", "poster_path",
                "poster_origin", "overview", "duration_seconds", "added_at",
                "provider", "provider_id", "rating", "genres", "top_cast",
                "runtime_minutes", "metadata_synced_at", "metadata_locked",
            ],
            Table::Episodes => &[
                "id", "show_id", "season", "episode", "title", "path",
                "duration_seconds", "added_at",
            ],
            Table::WatchHistory => &[
                "media_kind", "media_id", "progress_seconds", "duration_seconds",
                "watched", "last_watched_at",
            ],
            Table::MetadataJobs => &[
                "kind", "media_id", "enqueued_at", "attempts", "last_error",
                "next_attempt_at",
            ],
            Table::AppSettings => &["key", "value"],
        }
    }

    fn has_column(&self, candidate: &str) -> bool {
        self.columns().contains(&candidate)
    }
}

pub async fn list_rows(
    pool: &SqlitePool,
    table: Table,
    sort_column: Option<String>,
    direction: Option<String>,
) -> AppResult<Vec<Map<String, Value>>> {
    let mut sql = format!("SELECT * FROM {}", table.name());

    if let Some(column) = sort_column.as_deref() {
        if !table.has_column(column) {
            return Err(AppError::Other(format!(
                "unknown column: {column}"
            )));
        }

        let dir = match direction.as_deref() {
            Some("desc") => "DESC",
            _ => "ASC",
        };
        sql.push_str(&format!(" ORDER BY {column} {dir}"));
    }

    let rows = sqlx::query(&sql).fetch_all(pool).await?;

    let mut output = Vec::with_capacity(rows.len());
    for row in rows {
        output.push(row_to_json(&row)?);
    }

    Ok(output)
}

pub async fn update_row(
    pool: &SqlitePool,
    table: Table,
    primary_key_values: Vec<Value>,
    patch: HashMap<String, Value>,
) -> AppResult<()> {
    if patch.is_empty() {
        return Ok(());
    }

    let pk_columns = table.primary_key();
    if primary_key_values.len() != pk_columns.len() {
        return Err(AppError::Other(format!(
            "primary key arity mismatch for {}: expected {}, got {}",
            table.name(),
            pk_columns.len(),
            primary_key_values.len()
        )));
    }

    for column in patch.keys() {
        if !table.has_column(column) {
            return Err(AppError::Other(format!(
                "unknown column: {column}"
            )));
        }
    }

    let assignments: Vec<String> = patch
        .keys()
        .map(|column| format!("{column} = ?"))
        .collect();

    let where_clause: Vec<String> = pk_columns
        .iter()
        .map(|column| format!("{column} = ?"))
        .collect();

    let sql = format!(
        "UPDATE {} SET {} WHERE {}",
        table.name(),
        assignments.join(", "),
        where_clause.join(" AND "),
    );

    let mut query = sqlx::query(&sql);
    for value in patch.values() {
        query = bind_value(query, value);
    }
    for value in &primary_key_values {
        query = bind_value(query, value);
    }

    query.execute(pool).await?;
    Ok(())
}

pub async fn delete_rows(
    pool: &SqlitePool,
    table: Table,
    primary_keys: Vec<Vec<Value>>,
) -> AppResult<()> {
    if primary_keys.is_empty() {
        return Ok(());
    }

    let pk_columns = table.primary_key();
    let where_clause: Vec<String> = pk_columns
        .iter()
        .map(|column| format!("{column} = ?"))
        .collect();
    let sql = format!(
        "DELETE FROM {} WHERE {}",
        table.name(),
        where_clause.join(" AND "),
    );

    let mut tx = pool.begin().await?;

    for primary_key_values in primary_keys {
        if primary_key_values.len() != pk_columns.len() {
            tx.rollback().await?;
            return Err(AppError::Other(format!(
                "primary key arity mismatch for {}: expected {}, got {}",
                table.name(),
                pk_columns.len(),
                primary_key_values.len()
            )));
        }

        let mut query = sqlx::query(&sql);
        for value in &primary_key_values {
            query = bind_value(query, value);
        }
        query.execute(&mut *tx).await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Returns the value of `label_column` for the row identified by `pk_value`
/// inside `table`. Used by the FK chip to render a friendly label.
pub async fn fk_label(
    pool: &SqlitePool,
    table: Table,
    label_column: String,
    pk_value: Value,
) -> AppResult<Option<String>> {
    if !table.has_column(&label_column) {
        return Err(AppError::Other(format!(
            "unknown column: {label_column}"
        )));
    }

    let pk_columns = table.primary_key();
    if pk_columns.len() != 1 {
        return Err(AppError::Other(format!(
            "fk_label only supports single-column primary keys; {} is composite",
            table.name()
        )));
    }

    let sql = format!(
        "SELECT {} FROM {} WHERE {} = ? LIMIT 1",
        label_column,
        table.name(),
        pk_columns[0],
    );

    let mut query = sqlx::query_scalar::<_, Option<String>>(&sql);
    query = bind_value_scalar(query, &pk_value);
    let label = query.fetch_optional(pool).await?.flatten();

    Ok(label)
}

fn bind_value<'a>(
    query: sqlx::query::Query<'a, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'a>>,
    value: &'a Value,
) -> sqlx::query::Query<'a, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'a>> {
    match value {
        Value::Null => query.bind(Option::<i64>::None),
        Value::Bool(boolean) => query.bind(if *boolean { 1i64 } else { 0i64 }),
        Value::Number(number) => {
            if let Some(integer) = number.as_i64() {
                query.bind(integer)
            } else {
                query.bind(number.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(string) => query.bind(string.as_str()),
        Value::Array(_) | Value::Object(_) => {
            // Frontend serialises JSON columns to a string before sending.
            // If we end up here, the caller didn't follow the convention.
            // Fall through to a JSON-encoded string for forgiveness rather
            // than panicking.
            query.bind(value.to_string())
        }
    }
}

fn bind_value_scalar<'a, T>(
    query: sqlx::query::QueryScalar<
        'a,
        sqlx::Sqlite,
        T,
        sqlx::sqlite::SqliteArguments<'a>,
    >,
    value: &'a Value,
) -> sqlx::query::QueryScalar<
    'a,
    sqlx::Sqlite,
    T,
    sqlx::sqlite::SqliteArguments<'a>,
> {
    match value {
        Value::Null => query.bind(Option::<i64>::None),
        Value::Bool(boolean) => query.bind(if *boolean { 1i64 } else { 0i64 }),
        Value::Number(number) => {
            if let Some(integer) = number.as_i64() {
                query.bind(integer)
            } else {
                query.bind(number.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(string) => query.bind(string.as_str()),
        Value::Array(_) | Value::Object(_) => query.bind(value.to_string()),
    }
}

fn row_to_json(row: &sqlx::sqlite::SqliteRow) -> AppResult<Map<String, Value>> {
    let mut output = Map::new();

    for column in row.columns() {
        let index = column.ordinal();
        let name = column.name().to_string();

        let value_ref = row
            .try_get_raw(index)
            .map_err(|error| AppError::Other(format!("row get raw: {error}")))?;
        let json_value = sqlite_value_to_json(row, index, value_ref.type_info().name())?;
        output.insert(name, json_value);
    }

    Ok(output)
}

fn sqlite_value_to_json(
    row: &sqlx::sqlite::SqliteRow,
    index: usize,
    type_name: &str,
) -> AppResult<Value> {
    let value_ref = row
        .try_get_raw(index)
        .map_err(|error| AppError::Other(format!("row get raw: {error}")))?;
    if value_ref.is_null() {
        return Ok(Value::Null);
    }

    match type_name {
        "INTEGER" => {
            let value: i64 = row
                .try_get(index)
                .map_err(|error| AppError::Other(format!("row get int: {error}")))?;
            Ok(Value::from(value))
        }
        "REAL" => {
            let value: f64 = row
                .try_get(index)
                .map_err(|error| AppError::Other(format!("row get real: {error}")))?;
            Ok(Value::from(value))
        }
        "BLOB" => Ok(Value::String("<blob>".to_string())),
        _ => {
            let value: Option<String> = row
                .try_get(index)
                .map_err(|error| AppError::Other(format!("row get text: {error}")))?;
            Ok(match value {
                Some(text) => Value::String(text),
                None => Value::Null,
            })
        }
    }
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

    async fn seed_library(pool: &SqlitePool) -> i64 {
        sqlx::query("INSERT INTO libraries (path, kind) VALUES ('/tmp/lib', 'mixed')")
            .execute(pool)
            .await
            .unwrap();
        sqlx::query_scalar::<_, i64>("SELECT last_insert_rowid()")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn list_rows_returns_json_maps() {
        let pool = fresh_pool().await;
        seed_library(&pool).await;

        let rows = list_rows(&pool, Table::Libraries, None, None).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("path").unwrap(), &Value::String("/tmp/lib".into()));
        assert_eq!(rows[0].get("kind").unwrap(), &Value::String("mixed".into()));
        assert!(matches!(rows[0].get("id").unwrap(), Value::Number(_)));
    }

    #[tokio::test]
    async fn update_row_patches_only_listed_columns() {
        let pool = fresh_pool().await;
        let library_id = seed_library(&pool).await;

        let mut patch = HashMap::new();
        patch.insert("kind".to_string(), Value::String("series".to_string()));

        update_row(
            &pool,
            Table::Libraries,
            vec![Value::from(library_id)],
            patch,
        )
        .await
        .unwrap();

        let kind: String =
            sqlx::query_scalar("SELECT kind FROM libraries WHERE id = ?1")
                .bind(library_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(kind, "series");
    }

    #[tokio::test]
    async fn update_row_rejects_unknown_column() {
        let pool = fresh_pool().await;
        let library_id = seed_library(&pool).await;

        let mut patch = HashMap::new();
        patch.insert("not_a_real_column".to_string(), Value::Null);

        let result = update_row(
            &pool,
            Table::Libraries,
            vec![Value::from(library_id)],
            patch,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delete_rows_handles_composite_pk() {
        let pool = fresh_pool().await;

        sqlx::query(
            "INSERT INTO watch_history (media_kind, media_id, progress_seconds, watched)
             VALUES ('movie', 1, 60, 0)",
        )
        .execute(&pool)
        .await
        .unwrap();

        delete_rows(
            &pool,
            Table::WatchHistory,
            vec![vec![Value::String("movie".into()), Value::from(1i64)]],
        )
        .await
        .unwrap();

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM watch_history")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn fk_label_returns_path() {
        let pool = fresh_pool().await;
        let library_id = seed_library(&pool).await;

        let label = fk_label(
            &pool,
            Table::Libraries,
            "path".to_string(),
            Value::from(library_id),
        )
        .await
        .unwrap();
        assert_eq!(label, Some("/tmp/lib".to_string()));
    }

    #[tokio::test]
    async fn null_values_round_trip_as_json_null() {
        let pool = fresh_pool().await;
        let library_id = seed_library(&pool).await;

        sqlx::query("INSERT INTO shows (library_id, title, folder_path, fingerprint) VALUES (?1, 'Test', '/t', 't')")
            .bind(library_id)
            .execute(&pool)
            .await
            .unwrap();

        let rows = list_rows(&pool, Table::Shows, None, None).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("year"), Some(&Value::Null));
    }

    #[tokio::test]
    async fn bool_in_patch_writes_as_one() {
        let pool = fresh_pool().await;
        let library_id = seed_library(&pool).await;

        sqlx::query("INSERT INTO shows (library_id, title, folder_path, fingerprint) VALUES (?1, 'X', '/x', 'x')")
            .bind(library_id)
            .execute(&pool)
            .await
            .unwrap();
        let show_id: i64 = sqlx::query_scalar("SELECT last_insert_rowid()")
            .fetch_one(&pool)
            .await
            .unwrap();

        let mut patch = HashMap::new();
        patch.insert("metadata_locked".to_string(), Value::Bool(true));

        update_row(
            &pool,
            Table::Shows,
            vec![Value::from(show_id)],
            patch,
        )
        .await
        .unwrap();

        let stored: i64 =
            sqlx::query_scalar("SELECT metadata_locked FROM shows WHERE id = ?1")
                .bind(show_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(stored, 1);
    }
}
