use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::config::schema::PackageStatus;
use crate::tui::App;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Check results
            Constraint::Length(3), // Summary
            Constraint::Length(3), // Keybindings
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![Span::styled(
        " Doctor Check ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Check results
    let mut items: Vec<ListItem> = Vec::new();
    let mut issues = 0;

    // Package checks
    for pkg_state in &app.packages {
        let (icon, color, text) = match &pkg_state.status {
            PackageStatus::Installed { version } => (
                "OK",
                Color::Green,
                format!("{} ({})", pkg_state.package.id, version),
            ),
            PackageStatus::Outdated { current, available } => {
                issues += 1;
                (
                    "UP",
                    Color::Yellow,
                    format!("{} ({} -> {})", pkg_state.package.id, current, available),
                )
            }
            PackageStatus::NotInstalled => {
                issues += 1;
                (
                    "XX",
                    Color::Red,
                    format!("{} not installed", pkg_state.package.id),
                )
            }
            PackageStatus::Unknown => {
                issues += 1;
                (
                    "??",
                    Color::DarkGray,
                    format!("{} unknown", pkg_state.package.id),
                )
            }
        };

        items.push(
            ListItem::new(format!("  [{}] {}", icon, text)).style(Style::default().fg(color)),
        );
    }

    // Config checks
    for cfg in &app.profile.configs {
        let dest = std::path::Path::new(&cfg.dest);
        if dest.exists() {
            items.push(
                ListItem::new(format!("  [OK] Config: {}", cfg.dest))
                    .style(Style::default().fg(Color::Green)),
            );
        } else {
            issues += 1;
            items.push(
                ListItem::new(format!("  [XX] Config missing: {}", cfg.dest))
                    .style(Style::default().fg(Color::Red)),
            );
        }
    }

    // Env checks
    for (key, expected) in &app.profile.env {
        match std::env::var(key) {
            Ok(val) if val == *expected => {
                items.push(
                    ListItem::new(format!("  [OK] Env: {} = {}", key, expected))
                        .style(Style::default().fg(Color::Green)),
                );
            }
            Ok(val) => {
                issues += 1;
                items.push(
                    ListItem::new(format!(
                        "  [!!] Env: {} = {} (expected {})",
                        key, val, expected
                    ))
                    .style(Style::default().fg(Color::Yellow)),
                );
            }
            Err(_) => {
                issues += 1;
                items.push(
                    ListItem::new(format!("  [XX] Env: {} not set", key))
                        .style(Style::default().fg(Color::Red)),
                );
            }
        }
    }

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" Results "));
    frame.render_widget(list, chunks[1]);

    // Summary
    let summary_text = if issues == 0 {
        Span::styled(
            " All checks passed!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            format!(" {} issue(s) found", issues),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )
    };
    let summary = Paragraph::new(Line::from(vec![summary_text]))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(summary, chunks[2]);

    // Keybindings
    let keys = Paragraph::new(Line::from(vec![
        Span::styled(" Esc", Style::default().fg(Color::Cyan)),
        Span::raw(": back"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(keys, chunks[3]);
}
