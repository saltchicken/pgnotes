use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::Backend};
use std::{io, path::Path, process::Command};

pub fn open_editor<B: Backend + io::Write>(
    terminal: &mut Terminal<B>,
    file_path: &Path,
) -> io::Result<bool> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    // Open the editor
    let status = Command::new(editor).arg(file_path).status()?;

    enable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture
    )?;
    terminal.clear()?;

    Ok(status.success())
}
