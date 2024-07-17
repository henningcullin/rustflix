use crate::database::create_connection;

use super::Film;

pub fn get_all_films() -> Result<Vec<Film>, rusqlite::Error> {
    let conn = create_connection()?;

    let mut stmt = conn.prepare("SELECT id, title, file, link FROM films")?;
    let films: Vec<Film> = stmt
        .query_map([], |row| {
            Ok(Film {
                id: row.get(0)?,
                title: row.get(1)?,
                file: row.get(2)?,
                link: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(films)
}
