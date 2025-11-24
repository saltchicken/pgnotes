use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind, read},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, stdout};

mod config;
mod db;
mod editor;
mod events;
mod state;
mod ui;

use self::{config::Config, db::Database, events::handle_key_event, state::AppState, ui::ui};

pub struct App {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    state: AppState,
    database: Database,
}

impl App {
    pub fn new() -> io::Result<Self> {
        // 1. Load Config
        let config = Config::new();

        // 2. Init Database (Wrapped)
        let mut database = Database::new(&config.database_url)?;

        // 3. Init State (Pass DB info to state if needed, or just editor cmd)
        let editor_cmd = config.get_editor_command();
        let mut state = AppState::new(config.database_url.clone(), editor_cmd);

        // Initial data fetch
        state.refresh_notes(&mut database)?;

        // 4. Init Terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            state,
            database,
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            self.terminal.draw(|f| ui(f, &mut self.state))?;

            if let Event::Key(key) = read()? {
                if key.kind == KeyEventKind::Press {
                    // Pass specific subsystems to event handler
                    let should_continue = handle_key_event(
                        key,
                        &mut self.state,
                        &mut self.database,
                        &mut self.terminal,
                    )?;

                    if !should_continue {
                        break;
                    }
                }
            }
        }

        // Cleanup on exit
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;

        Ok(())
    }
}
