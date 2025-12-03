use crate::app::db::Database;
use ratatui::widgets::ListState;
use std::collections::HashSet;
use std::io;

#[derive(Debug, Clone)]
pub struct Note {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum TagFilter {
    All,
    Untagged,
    Specific(String),
}

impl std::fmt::Display for TagFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagFilter::All => write!(f, "All Notes"),
            TagFilter::Untagged => write!(f, "Untagged"),
            TagFilter::Specific(t) => write!(f, "#{}", t),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InputMode {
    Normal,
    EditingFilename,
    EditingTags,
    ConfirmingDelete,
    RenamingScript,
    SelectingTagFilter,
    ShowHelp,
}

pub struct AppState {
    pub all_notes: Vec<Note>,
    pub notes: Vec<Note>,
    pub list_state: ListState,
    pub status_message: String,
    pub script_content_preview: String,
    pub input_mode: InputMode,
    pub filename_input: String,
    pub help_message: String,
    pub editor_cmd: String,

    pub active_filter: TagFilter,
    pub available_filters: Vec<TagFilter>,
    pub filter_list_state: ListState,
}

impl AppState {
    pub fn new(db_url: String, editor_cmd: String) -> Self {
        let help_message = format!(
            "Welcome to Postgres Notes!\n\nDatabase: {}\n\n--- Keybinds ---\n'j'/'k'        : Navigate notes\n'Enter'/'e'    : Edit selected note\n'a'            : Add a new note\n'd'            : Delete selected note\n'r'            : Rename selected note\n't'            : Edit tags for note\n'Shift+t'      : Filter by Tag ‼️\n'?'            : Toggle help\n'q'            : Quit",
            db_url
        );

        Self {
            all_notes: Vec::new(),
            notes: Vec::new(),
            list_state: ListState::default(),
            status_message: "Welcome! Press '?' for help.".to_string(),
            script_content_preview: "".to_string(),
            input_mode: InputMode::Normal,
            filename_input: String::new(),
            help_message,
            editor_cmd,

            active_filter: TagFilter::All,
            available_filters: Vec::new(),
            filter_list_state: ListState::default(),
        }
    }

    pub fn refresh_notes(&mut self, db: &mut Database) -> io::Result<()> {
        match db.get_all_notes() {
            Ok(fetched_notes) => {
                self.all_notes = fetched_notes;

                self.apply_current_filter();

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

    pub fn apply_current_filter(&mut self) {
        self.notes = self
            .all_notes
            .iter()
            .filter(|n| match &self.active_filter {
                TagFilter::All => true,
                TagFilter::Untagged => n.tags.is_empty(),
                TagFilter::Specific(tag) => n.tags.contains(tag),
            })
            .cloned()
            .collect();

        self.notes.sort_by(|a, b| a.title.cmp(&b.title));
    }

    pub fn open_tag_selector(&mut self) {
        // 1. Collect unique tags
        let mut unique_tags: HashSet<String> = HashSet::new();
        for note in &self.all_notes {
            for tag in &note.tags {
                if !tag.is_empty() {
                    unique_tags.insert(tag.clone());
                }
            }
        }

        // 2. Sort tags
        let mut sorted_tags: Vec<String> = unique_tags.into_iter().collect();
        sorted_tags.sort();

        // 3. Build filter options
        self.available_filters = vec![TagFilter::All, TagFilter::Untagged];
        for tag in sorted_tags {
            self.available_filters.push(TagFilter::Specific(tag));
        }

        // 4. Set state
        self.input_mode = InputMode::SelectingTagFilter;
        self.filter_list_state.select(Some(0));
        self.set_status("Select tag to filter. [Enter] confirm, [Esc] cancel.".to_string());
    }

    pub fn next_filter(&mut self) {
        if self.available_filters.is_empty() {
            return;
        }
        let i = match self.filter_list_state.selected() {
            Some(i) => {
                if i >= self.available_filters.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.filter_list_state.select(Some(i));
    }

    pub fn previous_filter(&mut self) {
        if self.available_filters.is_empty() {
            return;
        }
        let i = match self.filter_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.available_filters.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.filter_list_state.select(Some(i));
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


            self.script_content_preview = note.content.clone();
        } else {
            self.script_content_preview = "No notes found.".to_string();
        }
    }
}