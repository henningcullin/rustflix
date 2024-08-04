enum Gender {
    Male,
    Female,
}

enum Country {
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

struct Person {
    id: u32,
    imdb_id: String,
    avatar: Option<String>,
    age: Option<u32>,
    gender: Option<Gender>,
    birthplace: Option<Country>,
}
