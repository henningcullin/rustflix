use src_macro::Fields;

use crate::persons::Person;

#[derive(Fields)]
pub struct Character {
    pub id: u32,
    pub actor: Person,
    pub description: String,
}
