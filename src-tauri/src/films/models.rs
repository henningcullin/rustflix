use serde::Serialize;

#[derive(Serialize)]
pub struct Film {
    pub id: u32,
    pub title: Option<String>,
    pub file: String,
    pub link: Option<String>,
}
