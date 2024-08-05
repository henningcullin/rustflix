use src_macro::Fields;

pub enum Gender {
    Male,
    Female,
}

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

#[derive(Fields)]
pub struct Person {
    pub id: u32,
    pub imdb_id: String,
    pub avatar: Option<String>,
    pub age: Option<u32>,
    pub gender: Option<Gender>,
    pub birthplace: Option<Country>,
}
