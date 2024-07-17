use serde::Serialize;

#[derive(Serialize)]
pub struct Directory {
    pub id: u32,
    pub path: String,
}
