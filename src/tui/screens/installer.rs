use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Row, Table};
use ratatui::Frame;

use crate::tui::{App, TaskStatus};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Progress list
            Constraint::Length(3), // Progress bar
            Constraint::Length(3), // Status line
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " Installing ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("- "),
        Span::styled(&app.profile.name, Style::default().fg(Color::Yellow)),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Package progress list
    let rows: Vec<Row> = app
        .packages
        .iter()
        .filter(|p| p.selected)
        .map(|pkg_state| {
            let (symbol, status_text, color) = match &pkg_state.task_status {
                TaskStatus::Idle => (".", "waiting...".to_string(), Color::DarkGray),
                TaskStatus::Checking => ("~", "checking...".to_string(), Color::Yellow),
                TaskStatus::Installing => ("~", "installing...".to_string(), Color::Cyan),
                TaskStatus::Upgrading => ("~", "upgrading...".to_string(), Color::Cyan),
                TaskStatus::Done => ("v", "done".to_string(), Color::Green),
                TaskStatus::Failed(msg) => ("x", format!("failed: {}", msg), Color::Red),
            };

            Row::new(vec![
                symbol.to_string(),
                pkg_state.package.id.clone(),
                status_text,
            ])
            .style(Style::default().fg(color))
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Min(25),
            Constraint::Min(30),
        ],
    )
    .block(Block::default().borders(Borders::ALL).title(" Progress "));

    frame.render_widget(table, chunks[1]);

    // Progress bar
    let total = app.install_total.max(1);
    let done = app.install_complete + app.install_failed;
    let ratio = done as f64 / total as f64;
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Cyan))
        .ratio(ratio.min(1.0))
        .label(format!("{}/{}", done, total));
    frame.render_widget(gauge, chunks[2]);

    // Status line
    let in_progress = app.in_progress_count();
    let status = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {} complete", app.install_complete),
            Style::default().fg(Color::Green),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{} in progress", in_progress),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{} failed", app.install_failed),
            Style::default().fg(if app.install_failed > 0 {
                Color::Red
            } else {
                Color::DarkGray
            }),
        ),
        if done >= total {
            Span::styled(
                "  [Press Esc for summary]",
                Style::default().fg(Color::Yellow),
            )
        } else {
            Span::raw("")
        },
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(status, chunks[3]);
}
