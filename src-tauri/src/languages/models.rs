use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Language {
    pub id: u32,
    pub name: String,
}
