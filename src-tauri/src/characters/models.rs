use serde::Serialize;
use src_macro::Fields;

use crate::persons::Person;

#[derive(Debug, Serialize, Fields)]
pub struct Character {
    pub id: u32,
    pub actor: Person,
    pub description: String,
}

impl Character {
    pub fn from_parts<'a, I>(parts: &mut I) -> Option<Character>
    where
        I: Iterator<Item = &'a str>,
    {
        Some(Character {
            id: parts.next()?.parse().ok()?,
            description: parts.next()?.to_string(),
            actor: Person::from_parts(parts)?,
        })
    }
}
