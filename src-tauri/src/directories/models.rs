use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Directory {
    pub id: u32,
    pub path: String,
}
