use super::{actions, Person};
use tauri::command;

#[command]
pub fn get_all_persons() -> Vec<Person> {
    match actions::get_all_persons() {
        Ok(persons) => persons,
        Err(_) => vec![],
    }
}
