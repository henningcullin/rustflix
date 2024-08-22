use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Genre {
    pub id: u32,
    pub name: String,
}
