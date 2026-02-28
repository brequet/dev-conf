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
            Constraint::Length(5), // Stats
            Constraint::Min(10),   // Messages
            Constraint::Length(3), // Keybindings
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![Span::styled(
        " Installation Summary ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Stats
    let stats = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            format!("  Installed: {}", app.install_complete),
            Style::default().fg(Color::Green),
        )]),
        Line::from(vec![Span::styled(
            format!("  Failed:    {}", app.install_failed),
            Style::default().fg(if app.install_failed > 0 {
                Color::Red
            } else {
                Color::DarkGray
            }),
        )]),
        Line::from(vec![Span::styled(
            format!("  Total:     {}", app.install_total),
            Style::default().fg(Color::White),
        )]),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Stats "));
    frame.render_widget(stats, chunks[1]);

    // Messages
    let items: Vec<ListItem> = app
        .summary_messages
        .iter()
        .map(|msg| {
            let color = if msg.starts_with("Failed") {
                Color::Red
            } else if msg.starts_with("Installed") {
                Color::Green
            } else {
                Color::White
            };
            ListItem::new(format!("  {}", msg)).style(Style::default().fg(color))
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" Log "));
    frame.render_widget(list, chunks[2]);

    // Keybindings
    let keys = Paragraph::new(Line::from(vec![
        Span::styled(" Enter/Esc", Style::default().fg(Color::Cyan)),
        Span::raw(": back to dashboard"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(keys, chunks[3]);
}
