use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::Backend};
use std::{io, path::Path, process::Command};

/// Opens an external editor (vim, nano, etc.) for the given file path.
/// Handles the terminal state transitions required to exit and re-enter the TUI.
pub fn open_editor<B: Backend + io::Write>(
    terminal: &mut Terminal<B>,
    file_path: &Path,
    editor_cmd: &str,
) -> io::Result<bool> {
    // 1. Suspend TUI state
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), DisableMouseCapture)?;
    terminal.show_cursor()?;

    // 2. Run the external editor process
    // We use .status() to wait for the child process to finish
    let status = Command::new(editor_cmd).arg(file_path).status();

    // 3. Restore TUI state (regardless of editor success)
    enable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture
    )?;
    terminal.clear()?; // Force a full redraw to clear artifacts

    // 4. Return success status
    match status {
        Ok(s) => Ok(s.success()),
        Err(e) => {
            // If the editor command itself failed to launch (e.g., command not found)
            // We return false so the app can display an error message
            eprintln!("Failed to open editor: {}", e);
            Ok(false)
        }
    }
}
