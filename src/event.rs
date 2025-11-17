// src/event.rs
use crate::{
    app::{App, InputMode},
    db,
    editor::open_editor,
};
use crossterm::event::{KeyCode, KeyEvent}; // ‼️ Removed KeyModifiers
use postgres::Client;
use ratatui::{Terminal, backend::Backend};
use std::{fs, io}; // ‼️ Removed Path

fn edit_note_in_external_editor<B: Backend + io::Write>(
    app: &mut App,
    client: &mut Client,
    terminal: &mut Terminal<B>,
) -> io::Result<()> {
    // ‼️ Fix: Extract ID and content first to end borrow of `app`
    let selection = app.get_selected_note().map(|n| (n.id, n.content.clone()));

    if let Some((id, content)) = selection {
        // 1. Create temp file
        let temp_dir = std::env::temp_dir();
        let temp_file_path = temp_dir.join(format!("pgnote_{}.md", id));
        fs::write(&temp_file_path, &content)?;

        // 2. Open editor
        let success = open_editor(terminal, &temp_file_path)?;

        if success {
            // 3. Read content back
            let new_content = fs::read_to_string(&temp_file_path)?;

            // 4. Update DB
            if let Err(e) = db::update_note_content(client, id, &new_content) {
                app.set_status(format!("Error saving note: {}", e));
            } else {
                app.set_status("Note saved.".to_string());
            }
        } else {
            app.set_status("Editor exited with error.".to_string());
        }

        // 5. Cleanup
        let _ = fs::remove_file(temp_file_path);

        // 6. Refresh app state
        app.refresh_notes(client)?;
    }
    Ok(())
}

pub fn handle_key_event<B: Backend + io::Write>(
    key: KeyEvent,
    app: &mut App,
    client: &mut Client,
    terminal: &mut Terminal<B>,
) -> io::Result<bool> {
    match app.input_mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('q') => return Ok(false),
            KeyCode::Char('j') => app.next(),
            KeyCode::Char('k') => app.previous(),
            KeyCode::Enter | KeyCode::Char('e') => {
                edit_note_in_external_editor(app, client, terminal)?;
            }
            KeyCode::Char('a') => {
                app.input_mode = InputMode::EditingFilename;
                app.filename_input.clear();
                app.set_status(
                    "Enter new note title. Press [Enter] to confirm, [Esc] to cancel.".to_string(),
                );
            }
            KeyCode::Char('d') => {
                // ‼️ Fix: Clone title to avoid holding a reference to app
                let selection = app.get_selected_note().map(|n| n.title.clone());
                if let Some(title) = selection {
                    app.input_mode = InputMode::ConfirmingDelete;
                    app.set_status(format!("Delete '{}'? (y/n)", title));
                } else {
                    app.set_status("No note selected to delete.".to_string());
                }
            }
            KeyCode::Char('r') => {
                // ‼️ Fix: Clone title to avoid holding a reference to app
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
            KeyCode::Char('?') => {
                app.input_mode = InputMode::ShowHelp;
            }
            _ => {}
        },
        InputMode::EditingFilename => match key.code {
            KeyCode::Enter => {
                // ‼️ Fix: .to_string() creates an owned String, ending the borrow of app.filename_input
                let title = app.filename_input.trim().to_string();
                if title.is_empty() {
                    app.input_mode = InputMode::Normal;
                    app.set_status("New note cancelled.".to_string());
                } else {
                    match db::create_note(client, &title) {
                        Ok(_) => {
                            app.set_status(format!("Note '{}' created.", title));
                            app.refresh_notes(client)?;
                            // Jump to the new note
                            if let Some(idx) = app.notes.iter().position(|n| n.title == title) {
                                app.list_state.select(Some(idx));
                                app.update_preview();
                                edit_note_in_external_editor(app, client, terminal)?;
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
        InputMode::ConfirmingDelete => match key.code {
            KeyCode::Char('y') => {
                // ‼️ Fix: Get ID first
                let selection = app.get_selected_note().map(|n| (n.id, n.title.clone()));
                if let Some((id, title)) = selection {
                    match db::delete_note(client, id) {
                        Ok(_) => {
                            app.set_status(format!("Note '{}' deleted.", title));
                            app.refresh_notes(client)?;
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
                // ‼️ Fix: .to_string() to own the data
                let new_title = app.filename_input.trim().to_string();
                if new_title.is_empty() {
                    app.input_mode = InputMode::Normal;
                    app.set_status("Rename cancelled.".to_string());
                } else {
                    // ‼️ Fix: Get ID first
                    let selection = app.get_selected_note().map(|n| n.id);
                    if let Some(id) = selection {
                        match db::rename_note(client, id, &new_title) {
                            Ok(_) => {
                                app.set_status("Note renamed.".to_string());
                                app.refresh_notes(client)?;
                                // Restore selection
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
        InputMode::ShowHelp => match key.code {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
                app.input_mode = InputMode::Normal;
            }
            _ => {}
        },
    }
    Ok(true)
}

