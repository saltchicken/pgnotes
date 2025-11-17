use postgres::{Client, Error};


#[derive(Debug, Clone)]
pub struct Note {
    pub id: i32,
    pub title: String,
    pub content: String,
}


pub fn init_db(client: &mut Client) -> Result<(), Error> {
    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id SERIAL PRIMARY KEY,
            title TEXT UNIQUE NOT NULL,
            content TEXT
        )",
    )?;
    Ok(())
}


pub fn get_all_notes(client: &mut Client) -> Result<Vec<Note>, Error> {
    let mut notes = Vec::new();
    for row in client.query("SELECT id, title, content FROM notes ORDER BY title", &[])? {
        notes.push(Note {
            id: row.get(0),
            title: row.get(1),
            content: row.get(2),
        });
    }
    Ok(notes)
}


pub fn create_note(client: &mut Client, title: &str) -> Result<(), Error> {
    client.execute(
        "INSERT INTO notes (title, content) VALUES ($1, '')",
        &[&title],
    )?;
    Ok(())
}


pub fn update_note_content(client: &mut Client, id: i32, content: &str) -> Result<(), Error> {
    client.execute(
        "UPDATE notes SET content = $1 WHERE id = $2",
        &[&content, &id],
    )?;
    Ok(())
}


pub fn rename_note(client: &mut Client, id: i32, new_title: &str) -> Result<(), Error> {
    client.execute(
        "UPDATE notes SET title = $1 WHERE id = $2",
        &[&new_title, &id],
    )?;
    Ok(())
}


pub fn delete_note(client: &mut Client, id: i32) -> Result<(), Error> {
    client.execute("DELETE FROM notes WHERE id = $1", &[&id])?;
    Ok(())
}