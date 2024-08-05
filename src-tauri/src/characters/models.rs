use serde::Serialize;
use src_macro::Fields;

use crate::persons::Person;

#[derive(Debug, Serialize, Fields)]
pub struct Character {
    pub id: u32,
    pub actor: Person,
    pub description: String,
}
