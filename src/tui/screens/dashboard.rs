use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use ratatui::Frame;

use crate::config::schema::{PackageSource, PackageStatus};
use crate::tui::{App, TaskStatus};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Package table
            Constraint::Length(3), // Keybindings
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " devconf ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("- "),
        Span::styled(
            &app.profile.name,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Package table
    let header = Row::new(vec!["Source", "Package", "Status", "Version"])
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

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

            let (symbol, status_text, version, color) = match &pkg_state.task_status {
                TaskStatus::Checking => {
                    ("~", "checking...".to_string(), String::new(), Color::Yellow)
                }
                TaskStatus::Installing => (
                    "~",
                    "installing...".to_string(),
                    String::new(),
                    Color::Yellow,
                ),
                TaskStatus::Upgrading => (
                    "~",
                    "upgrading...".to_string(),
                    String::new(),
                    Color::Yellow,
                ),
                TaskStatus::Done => ("v", "done".to_string(), String::new(), Color::Green),
                TaskStatus::Failed(msg) => {
                    ("x", format!("failed: {}", msg), String::new(), Color::Red)
                }
                TaskStatus::Idle => match &pkg_state.status {
                    PackageStatus::Installed { version } => {
                        ("v", "installed".to_string(), version.clone(), Color::Green)
                    }
                    PackageStatus::Outdated { current, available } => (
                        "^",
                        "outdated".to_string(),
                        format!("{} -> {}", current, available),
                        Color::Yellow,
                    ),
                    PackageStatus::NotInstalled => {
                        ("x", "missing".to_string(), "-".to_string(), Color::Red)
                    }
                    PackageStatus::Unknown => {
                        ("?", "unknown".to_string(), "-".to_string(), Color::DarkGray)
                    }
                },
            };

            let style = if i == app.selected_index {
                Style::default().bg(Color::DarkGray).fg(color)
            } else {
                Style::default().fg(color)
            };

            Row::new(vec![
                source.to_string(),
                pkg_state.package.id.clone(),
                format!("{} {}", symbol, status_text),
                version,
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Min(25),
            Constraint::Length(20),
            Constraint::Min(15),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Packages "));

    frame.render_widget(table, chunks[1]);

    // Keybindings
    let keys = Paragraph::new(Line::from(vec![
        Span::styled(" [i]", Style::default().fg(Color::Cyan)),
        Span::raw("nstall  "),
        Span::styled("[u]", Style::default().fg(Color::Cyan)),
        Span::raw("pgrade  "),
        Span::styled("[s]", Style::default().fg(Color::Cyan)),
        Span::raw("ync  "),
        Span::styled("[d]", Style::default().fg(Color::Cyan)),
        Span::raw("octor  "),
        Span::styled("[q]", Style::default().fg(Color::Cyan)),
        Span::raw("uit"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(keys, chunks[2]);
}
