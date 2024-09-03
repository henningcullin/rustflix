use serde::Serialize;

use crate::persons::Person;

#[derive(Debug, Serialize)]
pub struct Character {
    pub film_id: u32,
    pub description: String,
    pub actor: Person,
}

impl Character {
    pub fn from_parts<'a, I>(parts: &mut I) -> Option<Character>
    where
        I: Iterator<Item = &'a str>,
    {
        Some(Character {
            film_id: parts.next()?.parse().ok()?,
            description: parts.next()?.to_string(),
            actor: Person::from_parts(parts)?,
        })
    }
}
