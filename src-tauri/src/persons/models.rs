use serde::Serialize;
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

#[derive(Debug, Serialize)]
pub struct Person {
    pub id: u32,
    pub imdb_id: Option<String>,
    pub avatar: Option<String>,
    pub age: Option<u32>,
    pub gender: Option<Gender>,
    pub birthplace: Option<Country>,
}

impl Person {
    pub fn from_parts<'a, I>(parts: &mut I) -> Option<Person>
    where
        I: Iterator<Item = &'a str>,
    {
        Some(Person {
            id: parts.next()?.parse().ok()?,
            imdb_id: parts.next().map(|s| s.to_string()),
            avatar: parts.next().map(|s| s.to_string()),
            age: parts.next().and_then(|s| s.parse().ok()),
            gender: parts.next().and_then(|s| match s.parse::<i32>().ok()? {
                1 => Some(Gender::Male),
                2 => Some(Gender::Female),
                _ => None,
            }),
            birthplace: parts
                .next()
                .and_then(|s| s.parse::<i32>().ok())
                .map(map_birthplace),
        })
    }
}

fn map_birthplace(code: i32) -> Country {
    match code {
        1 => Country::Sweden,
        2 => Country::UnitedKingdom,
        3 => Country::Norway,
        4 => Country::UnitedStates,
        5 => Country::Canada,
        6 => Country::Mexico,
        7 => Country::Russia,
        8 => Country::France,
        9 => Country::Germany,
        10 => Country::Spain,
        11 => Country::Italy,
        12 => Country::Portugal,
        _ => unreachable!(),
    }
}
