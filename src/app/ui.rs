use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use super::state::{AppState, InputMode};

pub fn ui(f: &mut Frame, app: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(f.area());

    // --- Left Pane: Note List ---

    let items: Vec<ListItem> = app
        .notes
        .iter()
        .map(|note| {
            let label = if note.tags.is_empty() {
                note.title.clone()
            } else {
                // Show title + first tag or tag count indicator
                format!("{} [{}]", note.title, note.tags.join(","))
            };
            ListItem::new(label)
        })
        .collect();

    let list_title = if app.search_query.is_empty() {
        format!("Notes (Filter: {})", app.active_filter)
    } else {
        format!(
            "Search: '{}' (Filter: {})",
            app.search_query, app.active_filter
        )
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(list_title))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[0], &mut app.list_state);

    // --- Right Pane: Preview ---
    let preview_block = Block::default().borders(Borders::ALL).title("Note Content");
    let preview_text = Paragraph::new(app.script_content_preview.as_str())
        .block(preview_block)
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll, 0));

    f.render_widget(preview_text, chunks[1]);

    // --- Popup Windows ---
    match app.input_mode {
        InputMode::EditingFilename => {
            let area = centered_fixed_height_rect(50, 3, f.area());
            let input_text = format!("{}_", app.filename_input);
            let popup_block = Block::default()
                .title("New Note Title")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::LightBlue));
            let input_paragraph = Paragraph::new(input_text.as_str()).block(popup_block);
            f.render_widget(Clear, area);
            f.render_widget(input_paragraph, area);
        }
        InputMode::ConfirmingDelete => {
            let area = centered_fixed_height_rect(50, 3, f.area());
            let popup_block = Block::default()
                .title("Confirm Deletion")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Red).fg(Color::White));

            let popup_paragraph = Paragraph::new(app.status_message.as_str())
                .block(popup_block)
                .alignment(Alignment::Center);

            f.render_widget(Clear, area);
            f.render_widget(popup_paragraph, area);
        }
        InputMode::RenamingScript => {
            let area = centered_fixed_height_rect(50, 3, f.area());
            let input_text = format!("{}_", app.filename_input);
            let popup_block = Block::default()
                .title("Rename Note")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::LightYellow).fg(Color::Black));

            let input_paragraph = Paragraph::new(input_text.as_str()).block(popup_block);
            f.render_widget(Clear, area);
            f.render_widget(input_paragraph, area);
        }

        InputMode::EditingTags => {
            let area = centered_fixed_height_rect(50, 3, f.area());
            let input_text = format!("{}_", app.filename_input);
            let popup_block = Block::default()
                .title("Edit Tags (comma separated)")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::LightCyan).fg(Color::Black));

            let input_paragraph = Paragraph::new(input_text.as_str()).block(popup_block);
            f.render_widget(Clear, area);
            f.render_widget(input_paragraph, area);
        }

        InputMode::Searching => {
            let area = centered_fixed_height_rect(50, 3, f.area());
            let input_text = format!("{}_", app.search_query);
            let popup_block = Block::default()
                .title("Search Titles")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Blue).fg(Color::White));

            let input_paragraph = Paragraph::new(input_text.as_str()).block(popup_block);
            f.render_widget(Clear, area);
            f.render_widget(input_paragraph, area);
        }

        InputMode::ShowHelp => {
            // NOTE: Keep percentage here (15%) or switch to fixed if preferred. 15% is usually okay for help.
            let area = centered_rect(60, 15, f.area());
            let popup_block = Block::default().title("Help").borders(Borders::ALL);
            let popup_paragraph = Paragraph::new(app.help_message.as_str())
                .block(popup_block)
                .alignment(Alignment::Left);

            f.render_widget(Clear, area);
            f.render_widget(popup_paragraph, area);
        }

        InputMode::SelectingTagFilter => {
            // NOTE: Keep percentage here (50%) for the list
            let area = centered_rect(40, 50, f.area());
            let items: Vec<ListItem> = app
                .available_filters
                .iter()
                .map(|f| ListItem::new(format!("{}", f)))
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Filter by Tag")
                        .style(Style::default().bg(Color::DarkGray)),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");

            f.render_widget(Clear, area);
            f.render_stateful_widget(list, area, &mut app.filter_list_state);
        }
        InputMode::Normal => {}
    }
}

fn centered_fixed_height_rect(percent_x: u16, height_abs: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height_abs),
            Constraint::Fill(1),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn centered_rect(percent_x: u16, height_percent: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

