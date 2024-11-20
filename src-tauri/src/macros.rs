#[macro_export]
macro_rules! execute_and_return_id {
    ($conn:expr, $sql:expr, $params:expr) => {{
        let mut stmt = $conn.prepare($sql)?;
        let id: i32 = stmt.query_row($params, |row| row.get(0))?;
        Ok(id)
    }};
}
