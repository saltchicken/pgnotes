use crate::db::{self, Note};
use postgres::Client;
use ratatui::widgets::ListState;
use std::io;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InputMode {
    Normal,
    EditingFilename, // Used for creating new notes
    ConfirmingDelete,
    RenamingScript, // Used for renaming notes
    ShowHelp,
}
pub struct App {
    pub notes: Vec<Note>,
    pub list_state: ListState,
    pub status_message: String,
    pub script_content_preview: String,
    pub input_mode: InputMode,
    pub filename_input: String,
    pub help_message: String,
    pub editor_cmd: String,
}
impl App {
    pub fn new(client: &mut Client, db_url: &str, editor_cmd: String) -> io::Result<Self> {
        let help_message = format!(
            "Welcome to Postgres Notes!\n\nDatabase: {}\n\n--- Keybinds ---\n'j'/'k'        : Navigate notes\n'Enter'/'e'    : Edit selected note\n'a'            : Add a new note\n'd'            : Delete selected note\n'r'            : Rename selected note\n'?'            : Toggle help\n'q'            : Quit",
            db_url
        );
        let mut app = Self {
            notes: Vec::new(),
            list_state: ListState::default(),
            status_message: "Welcome! Press '?' for help.".to_string(),
            script_content_preview: "".to_string(),
            input_mode: InputMode::Normal,
            filename_input: String::new(),
            help_message,
            editor_cmd,
        };
        app.refresh_notes(client)?;
        Ok(app)
    }
    pub fn set_status(&mut self, message: String) {
        self.status_message = message;
    }
    pub fn refresh_notes(&mut self, client: &mut Client) -> io::Result<()> {
        match db::get_all_notes(client) {
            Ok(notes) => {
                self.notes = notes;
                // Validate selection
                let mut valid_selection_exists = false;
                if let Some(selected_index) = self.list_state.selected() {
                    valid_selection_exists = selected_index < self.notes.len();
                }
                if !valid_selection_exists {
                    if !self.notes.is_empty() {
                        self.list_state.select(Some(0));
                    } else {
                        self.list_state.select(None);
                    }
                }
                self.update_preview();
            }
            Err(e) => self.set_status(format!("DB Error: {}", e)),
        }
        Ok(())
    }
    pub fn get_selected_note(&self) -> Option<&Note> {
        self.list_state.selected().and_then(|i| self.notes.get(i))
    }
    pub fn next(&mut self) {
        if self.notes.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.notes.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.update_preview();
    }
    pub fn previous(&mut self) {
        if self.notes.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.notes.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.update_preview();
    }
    pub fn update_preview(&mut self) {
        if let Some(note) = self.get_selected_note() {
            self.script_content_preview = note.content.clone();
        } else {
            self.script_content_preview = "No notes found.".to_string();
        }
    }
}
