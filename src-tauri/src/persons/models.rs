use serde::Serialize;
use src_macro::Fields;

#[derive(Debug, Serialize)]
pub enum Gender {
    Male,
    Female,
}

#[derive(Debug, Serialize)]
pub enum Country {
    Sweden,
    UnitedKingdom,
    Norway,
    UnitedStates,
    Canada,
    Mexico,
    Russia,
    France,
    Germany,
    Spain,
    Italy,
    Portugal,
}

#[derive(Debug, Serialize, Fields)]
pub struct Person {
    pub id: u32,
    pub imdb_id: String,
    pub avatar: Option<String>,
    pub age: Option<u32>,
    pub gender: Option<Gender>,
    pub birthplace: Option<Country>,
}
