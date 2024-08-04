use src_macro::Select;

#[derive(Select)]
struct Character {
    id: u32,
    person_id: u32,
    film_id: u32,
    description: String,
}
