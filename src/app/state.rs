use crate::app::db::Database;
use ratatui::widgets::ListState;
use std::cmp::Ordering;
use std::io;

#[derive(Debug, Clone)]
pub struct Note {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InputMode {
    Normal,
    EditingFilename,
    EditingTags,
    ConfirmingDelete,
    RenamingScript,
    ShowHelp,
}

pub struct AppState {
    pub notes: Vec<Note>,
    pub list_state: ListState,
    pub status_message: String,
    pub script_content_preview: String,
    pub input_mode: InputMode,
    pub filename_input: String,
    pub help_message: String,
    pub editor_cmd: String,
    pub sort_by_tags: bool,
}

impl AppState {
    pub fn new(db_url: String, editor_cmd: String) -> Self {
        let help_message = format!(
            "Welcome to Postgres Notes!\n\nDatabase: {}\n\n--- Keybinds ---\n'j'/'k'        : Navigate notes\n'Enter'/'e'    : Edit selected note\n'a'            : Add a new note\n'd'            : Delete selected note\n'r'            : Rename selected note\n't'            : Edit tags for note ‼️\n's'            : Toggle sort (Title/Tags) ‼️\n'?'            : Toggle help\n'q'            : Quit",
            db_url
        );

        Self {
            notes: Vec::new(),
            list_state: ListState::default(),
            status_message: "Welcome! Press '?' for help.".to_string(),
            script_content_preview: "".to_string(),
            input_mode: InputMode::Normal,
            filename_input: String::new(),
            help_message,
            editor_cmd,
            sort_by_tags: false,
        }
    }

    pub fn refresh_notes(&mut self, db: &mut Database) -> io::Result<()> {
        match db.get_all_notes() {
            Ok(notes) => {
                self.notes = notes;

                self.sort_notes();

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


    pub fn sort_notes(&mut self) {
        if self.sort_by_tags {
            self.notes.sort_by(|a, b| {
                // Sort by tags first, then by title
                match a.tags.cmp(&b.tags) {
                    Ordering::Equal => a.title.cmp(&b.title),
                    other => other,
                }
            });
            self.set_status("Sorted by: Tags".to_string());
        } else {
            self.notes.sort_by(|a, b| a.title.cmp(&b.title));
            self.set_status("Sorted by: Title".to_string());
        }
    }

    pub fn toggle_sort(&mut self) {
        self.sort_by_tags = !self.sort_by_tags;
        self.sort_notes();
        // Reset selection to top after sort to avoid confusion
        if !self.notes.is_empty() {
            self.list_state.select(Some(0));
            self.update_preview();
        }
    }

    pub fn set_status(&mut self, message: String) {
        self.status_message = message;
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

            let tags_line = if note.tags.is_empty() {
                "No Tags".to_string()
            } else {
                format!("Tags: [{}]", note.tags.join(", "))
            };
            self.script_content_preview = format!("{}\n\n{}", tags_line, note.content);
        } else {
            self.script_content_preview = "No notes found.".to_string();
        }
    }
}