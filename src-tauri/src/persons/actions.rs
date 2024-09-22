use crate::{database::create_connection, error::AppError};

use super::Person;

pub fn get_all_persons() -> Result<Vec<Person>, AppError> {
    let conn = create_connection()?;

    let mut stmt = conn.prepare(
        r#"--sql
        SELECT 
            id,  
            imdb_id,  
            name,  
            age,  
            gender,  
            birthplace
        FROM
            persons
        ORDER BY
            name ASC"#,
    )?;

    let persons = stmt
        .query_map([], |row| Person::from_row(row))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(persons)
}
