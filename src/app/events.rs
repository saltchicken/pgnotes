use crate::app::{
    db::Database,
    editor::open_editor,
    state::{AppState, InputMode},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::Backend};
use std::{fs, io};

fn edit_note_in_external_editor<B: Backend + io::Write>(
    app: &mut AppState,
    db: &mut Database,
    terminal: &mut Terminal<B>,
) -> io::Result<()> {
    // The ID is correct, so database operations will target the correct note.
    let selection = app.get_selected_note().map(|n| (n.id, n.content.clone()));

    if let Some((id, content)) = selection {
        let temp_dir = std::env::temp_dir();
        let temp_file_path = temp_dir.join(format!("pgnote_{}.md", id));
        fs::write(&temp_file_path, &content)?;

        let success = open_editor(terminal, &temp_file_path, &app.editor_cmd)?;

        if success {
            let new_content = fs::read_to_string(&temp_file_path)?;

            if let Err(e) = db.update_note_content(id, &new_content) {
                app.set_status(format!("Error saving note: {}", e));
            } else {
                app.set_status("Note saved.".to_string());
            }
        } else {
            app.set_status("Editor exited with error.".to_string());
        }

        let _ = fs::remove_file(temp_file_path);
        app.refresh_notes(db)?;
    }
    Ok(())
}

pub fn handle_key_event<B: Backend + io::Write>(
    key: KeyEvent,
    app: &mut AppState,
    db: &mut Database,
    terminal: &mut Terminal<B>,
) -> io::Result<bool> {
    match app.input_mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('q') => return Ok(false),
            KeyCode::Char('j') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    app.scroll_preview_down();
                } else {
                    app.next();
                }
            }
            KeyCode::Char('k') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    app.scroll_preview_up();
                } else {
                    app.previous();
                }
            }

            KeyCode::Down => app.scroll_preview_down(),
            KeyCode::Up => app.scroll_preview_up(),

            KeyCode::Enter | KeyCode::Char('e') => {
                edit_note_in_external_editor(app, db, terminal)?;
            }
            KeyCode::Char('a') => {
                app.input_mode = InputMode::EditingFilename;
                app.filename_input.clear();
                app.set_status(
                    "Enter new note title. Press [Enter] to confirm, [Esc] to cancel.".to_string(),
                );
            }
            KeyCode::Char('d') => {
                let selection = app.get_selected_note().map(|n| n.title.clone());
                if let Some(title) = selection {
                    app.input_mode = InputMode::ConfirmingDelete;
                    app.set_status(format!("Delete '{}'? (y/n)", title));
                } else {
                    app.set_status("No note selected to delete.".to_string());
                }
            }
            KeyCode::Char('r') => {
                let selection = app.get_selected_note().map(|n| n.title.clone());
                if let Some(title) = selection {
                    app.input_mode = InputMode::RenamingScript;
                    app.filename_input = title;
                    app.set_status(
                        "Enter new title. Press [Enter] to confirm, [Esc] to cancel.".to_string(),
                    );
                } else {
                    app.set_status("No note selected to rename.".to_string());
                }
            }

            KeyCode::Char('x') => {
                if let Some(note) = app.get_selected_note() {
                    let new_status = !note.archived;
                    match db.update_archive_status(note.id, new_status) {
                        Ok(_) => {
                            let action = if new_status { "Archived" } else { "Unarchived" };
                            app.set_status(format!("Note '{}' {}.", note.title, action));
                            app.refresh_notes(db)?;
                        }
                        Err(e) => app.set_status(format!("Error updating archive status: {}", e)),
                    }
                }
            }

            KeyCode::Char('v') => {
                app.toggle_view_mode();
                app.apply_current_filter();
                // Select first if available
                if !app.notes.is_empty() {
                    app.list_state.select(Some(0));
                }
                app.update_preview();
                let view_name = match app.view_mode {
                    crate::app::state::ViewMode::Active => "Active Notes",
                    crate::app::state::ViewMode::Archived => "Archived Notes",
                };
                app.set_status(format!("Switched to {}", view_name));
            }

            KeyCode::Char('t') => {
                let current_tags = app.get_selected_note().map(|n| n.tags.join(", "));

                if let Some(tags) = current_tags {
                    app.input_mode = InputMode::EditingTags;
                    app.filename_input = tags; // Pre-fill with current tags
                    app.set_status(
                        "Edit tags (comma separated). [Enter] save, [Esc] cancel.".to_string(),
                    );
                } else {
                    app.set_status("No note selected.".to_string());
                }
            }

            KeyCode::Char('T') => {
                app.open_tag_selector();
            }

            KeyCode::Char('/') => {
                app.input_mode = InputMode::Searching;
                app.set_status(
                    "Search mode: Type to filter, [Enter] to keep filter, [Esc] to clear."
                        .to_string(),
                );
            }

            KeyCode::Char('?') => {
                app.input_mode = InputMode::ShowHelp;
            }
            _ => {}
        },

        InputMode::Searching => match key.code {
            KeyCode::Enter => {
                // Keep the filter applied, return to normal navigation
                app.input_mode = InputMode::Normal;
                app.set_status(format!("Search applied: '{}'", app.search_query));
            }
            KeyCode::Esc => {
                // Clear search and return to normal
                app.search_query.clear();
                app.apply_current_filter();
                app.input_mode = InputMode::Normal;
                app.set_status("Search cleared.".to_string());
                // Reset list to top
                if !app.notes.is_empty() {
                    app.list_state.select(Some(0));
                }
            }
            KeyCode::Backspace => {
                app.search_query.pop();
                app.apply_current_filter();
            }
            KeyCode::Char(c) => {
                app.search_query.push(c);
                app.apply_current_filter();
            }
            _ => {}
        },

        InputMode::EditingFilename => match key.code {
            KeyCode::Enter => {
                let title = app.filename_input.trim().to_string();
                if title.is_empty() {
                    app.input_mode = InputMode::Normal;
                    app.set_status("New note cancelled.".to_string());
                } else {
                    match db.create_note(&title) {
                        Ok(_) => {
                            app.set_status(format!("Note '{}' created.", title));
                            app.refresh_notes(db)?;

                            if let Some(idx) = app.notes.iter().position(|n| n.title == title) {
                                app.list_state.select(Some(idx));
                                app.update_preview();
                                edit_note_in_external_editor(app, db, terminal)?;
                            }
                        }
                        Err(e) => app.set_status(format!("Error creating note: {}", e)),
                    }
                    app.input_mode = InputMode::Normal;
                }
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                app.set_status("New note cancelled.".to_string());
            }
            KeyCode::Backspace => {
                app.filename_input.pop();
            }
            KeyCode::Char(c) => {
                app.filename_input.push(c);
            }
            _ => {}
        },

        InputMode::EditingTags => match key.code {
            KeyCode::Enter => {
                let tags: Vec<String> = app
                    .filename_input
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if let Some(note) = app.get_selected_note() {
                    match db.update_note_tags(note.id, &tags) {
                        Ok(_) => {
                            app.set_status("Tags updated.".to_string());
                            app.refresh_notes(db)?;
                        }
                        Err(e) => app.set_status(format!("Error updating tags: {}", e)),
                    }
                }
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                app.set_status("Tag editing cancelled.".to_string());
            }
            KeyCode::Backspace => {
                app.filename_input.pop();
            }
            KeyCode::Char(c) => {
                app.filename_input.push(c);
            }
            _ => {}
        },
        InputMode::ConfirmingDelete => match key.code {
            KeyCode::Char('y') => {
                let selection = app.get_selected_note().map(|n| (n.id, n.title.clone()));
                if let Some((id, title)) = selection {
                    match db.delete_note(id) {
                        Ok(_) => {
                            app.set_status(format!("Note '{}' deleted.", title));
                            app.refresh_notes(db)?;
                        }
                        Err(e) => app.set_status(format!("Error deleting note: {}", e)),
                    }
                }
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                app.set_status("Deletion cancelled.".to_string());
            }
            _ => {}
        },
        InputMode::RenamingScript => match key.code {
            KeyCode::Enter => {
                let new_title = app.filename_input.trim().to_string();
                if new_title.is_empty() {
                    app.input_mode = InputMode::Normal;
                    app.set_status("Rename cancelled.".to_string());
                } else {
                    let selection = app.get_selected_note().map(|n| n.id);
                    if let Some(id) = selection {
                        match db.rename_note(id, &new_title) {
                            Ok(_) => {
                                app.set_status("Note renamed.".to_string());
                                app.refresh_notes(db)?;
                                if let Some(idx) =
                                    app.notes.iter().position(|n| n.title == new_title)
                                {
                                    app.list_state.select(Some(idx));
                                }
                            }
                            Err(e) => app.set_status(format!("Error renaming note: {}", e)),
                        }
                    }
                    app.input_mode = InputMode::Normal;
                }
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                app.set_status("Rename cancelled.".to_string());
            }
            KeyCode::Backspace => {
                app.filename_input.pop();
            }
            KeyCode::Char(c) => {
                app.filename_input.push(c);
            }
            _ => {}
        },

        InputMode::SelectingTagFilter => match key.code {
            KeyCode::Char('j') => app.next_filter(),
            KeyCode::Char('k') => app.previous_filter(),
            KeyCode::Enter => {
                if let Some(idx) = app.filter_list_state.selected() {
                    if let Some(filter) = app.available_filters.get(idx).cloned() {
                        app.active_filter = filter.clone();
                        app.apply_current_filter();
                        // Reset list selection
                        if !app.notes.is_empty() {
                            app.list_state.select(Some(0));
                        } else {
                            app.list_state.select(None);
                        }
                        app.update_preview();
                        app.set_status(format!("Filter applied: {}", filter));
                    }
                }
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.input_mode = InputMode::Normal;
                app.set_status("Filter cancelled.".to_string());
            }
            _ => {}
        },
        InputMode::ShowHelp => match key.code {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
                app.input_mode = InputMode::Normal;
            }
            _ => {}
        },
    }
    Ok(true)
}