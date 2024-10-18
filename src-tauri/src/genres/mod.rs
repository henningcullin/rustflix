pub mod actions;
pub mod commands;
pub mod models;

pub use commands::{add_genre_to_film, get_all_genres, remove_genre_from_film};
pub use models::Genre;
