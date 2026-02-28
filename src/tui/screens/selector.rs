use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use ratatui::Frame;

use crate::config::schema::{PackageSource, PackageStatus};
use crate::tui::App;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Package list
            Constraint::Length(3), // Keybindings
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![Span::styled(
        " Select packages to install ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Package list with checkboxes
    let rows: Vec<Row> = app
        .packages
        .iter()
        .enumerate()
        .map(|(i, pkg_state)| {
            let source = match &pkg_state.package.source {
                PackageSource::Winget => "winget",
                PackageSource::Scoop => "scoop",
                PackageSource::Choco => "choco",
                PackageSource::GitHub(_) => "github",
            };

            let checkbox = if pkg_state.selected { "[x]" } else { "[ ]" };

            let (status_text, color) = match &pkg_state.status {
                PackageStatus::Installed { version } => {
                    (format!("v installed ({})", version), Color::Green)
                }
                PackageStatus::Outdated { current, available } => (
                    format!("^ upgrade {} -> {}", current, available),
                    Color::Yellow,
                ),
                PackageStatus::NotInstalled => ("x not installed".to_string(), Color::Red),
                PackageStatus::Unknown => ("? unknown".to_string(), Color::DarkGray),
            };

            let style = if i == app.selected_index {
                Style::default().bg(Color::DarkGray).fg(color)
            } else {
                Style::default().fg(color)
            };

            Row::new(vec![
                checkbox.to_string(),
                source.to_string(),
                pkg_state.package.id.clone(),
                status_text,
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Length(10),
            Constraint::Min(25),
            Constraint::Min(20),
        ],
    )
    .block(Block::default().borders(Borders::ALL).title(" Packages "));

    frame.render_widget(table, chunks[1]);

    // Keybindings
    let keys = Paragraph::new(Line::from(vec![
        Span::styled(" Space", Style::default().fg(Color::Cyan)),
        Span::raw(": toggle  "),
        Span::styled("a", Style::default().fg(Color::Cyan)),
        Span::raw(": all  "),
        Span::styled("n", Style::default().fg(Color::Cyan)),
        Span::raw(": none  "),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw(": confirm  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(": back"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(keys, chunks[2]);
}
