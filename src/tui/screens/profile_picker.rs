use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::tui::App;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Profile list
            Constraint::Length(3), // Keybindings
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![Span::styled(
        " Select Profile ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Profile list
    let items: Vec<ListItem> = app
        .available_profiles
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if i == app.profile_selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };
            let indicator = if name == &app.profile.name {
                " (active)"
            } else {
                ""
            };
            ListItem::new(format!("  {}{}", name, indicator)).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" Profiles "));
    frame.render_widget(list, chunks[1]);

    // Keybindings
    let keys = Paragraph::new(Line::from(vec![
        Span::styled(" Enter", Style::default().fg(Color::Cyan)),
        Span::raw(": select  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(": back"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(keys, chunks[2]);
}
