use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::Backend};
use std::{io, path::Path, process::Command};

pub fn open_editor<B: Backend + io::Write>(
    terminal: &mut Terminal<B>,
    file_path: &Path,
    editor_cmd: &str,
) -> io::Result<bool> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), DisableMouseCapture)?;
    terminal.show_cursor()?;

    let status = Command::new(editor_cmd).arg(file_path).status()?;

    enable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture
    )?;
    terminal.clear()?;

    Ok(status.success())
}
