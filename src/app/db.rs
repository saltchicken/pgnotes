use crate::app::state::Note;
use postgres::{Client, Error, NoTls}; // Access Note definition

pub struct Database {
    client: Client,
}

impl Database {
    pub fn new(db_url: &str) -> std::io::Result<Self> {
        let mut client = Client::connect(db_url, NoTls)
            .map_err(|e| std::io::Error::other(format!("DB connect error: {:#?}", e)))?;

        // Run init migration immediately
        client
            .batch_execute(
                "CREATE TABLE IF NOT EXISTS notes (
                id SERIAL PRIMARY KEY,
                title TEXT UNIQUE NOT NULL,
                content TEXT
            )",
            )
            .map_err(std::io::Error::other)?;

        Ok(Self { client })
    }

    pub fn get_all_notes(&mut self) -> Result<Vec<Note>, Error> {
        let mut notes = Vec::new();
        for row in self
            .client
            .query("SELECT id, title, content FROM notes ORDER BY title", &[])?
        {
            notes.push(Note {
                id: row.get(0),
                title: row.get(1),
                content: row.get(2),
            });
        }
        Ok(notes)
    }

    pub fn create_note(&mut self, title: &str) -> Result<(), Error> {
        self.client.execute(
            "INSERT INTO notes (title, content) VALUES ($1, '')",
            &[&title],
        )?;
        Ok(())
    }

    pub fn update_note_content(&mut self, id: i32, content: &str) -> Result<(), Error> {
        self.client.execute(
            "UPDATE notes SET content = $1 WHERE id = $2",
            &[&content, &id],
        )?;
        Ok(())
    }

    pub fn rename_note(&mut self, id: i32, new_title: &str) -> Result<(), Error> {
        self.client.execute(
            "UPDATE notes SET title = $1 WHERE id = $2",
            &[&new_title, &id],
        )?;
        Ok(())
    }

    pub fn delete_note(&mut self, id: i32) -> Result<(), Error> {
        self.client
            .execute("DELETE FROM notes WHERE id = $1", &[&id])?;
        Ok(())
    }
}

