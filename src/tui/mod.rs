pub mod screens;

use std::io;
use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;

use crate::config::schema::{Package, PackageStatus, ResolvedProfile};

/// Messages sent from background tasks to the TUI
#[derive(Debug, Clone)]
pub enum TaskUpdate {
    CheckStarted { index: usize },
    CheckComplete { index: usize, status: PackageStatus },
    InstallStarted { index: usize },
    InstallComplete { index: usize, success: bool, message: String },
}

/// State for a single package in the TUI
#[derive(Debug, Clone)]
pub struct PackageState {
    pub package: Package,
    pub status: PackageStatus,
    pub task_status: TaskStatus,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Idle,
    Checking,
    Installing,
    Upgrading,
    Done,
    Failed(String),
}

/// Which screen is currently active
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Dashboard,
    Selector,
    Installer,
    ProfilePicker,
    Doctor,
    Summary,
}

/// Main application state
pub struct App {
    pub profile: ResolvedProfile,
    pub packages: Vec<PackageState>,
    pub screen: Screen,
    pub selected_index: usize,
    pub should_quit: bool,
    pub available_profiles: Vec<String>,
    pub profile_selected_index: usize,
    pub install_complete: usize,
    pub install_failed: usize,
    pub install_total: usize,
    pub summary_messages: Vec<String>,
}

impl App {
    pub fn new(profile: ResolvedProfile, available_profiles: Vec<String>) -> Self {
        let packages: Vec<PackageState> = profile
            .packages
            .iter()
            .map(|p| PackageState {
                package: p.clone(),
                status: PackageStatus::Unknown,
                task_status: TaskStatus::Idle,
                selected: true,
            })
            .collect();

        App {
            profile,
            packages,
            screen: Screen::Dashboard,
            selected_index: 0,
            should_quit: false,
            available_profiles,
            profile_selected_index: 0,
            install_complete: 0,
            install_failed: 0,
            install_total: 0,
            summary_messages: Vec::new(),
        }
    }

    pub fn in_progress_count(&self) -> usize {
        self.packages
            .iter()
            .filter(|p| {
                matches!(
                    p.task_status,
                    TaskStatus::Checking | TaskStatus::Installing | TaskStatus::Upgrading
                )
            })
            .count()
    }
}

/// Initialize terminal for TUI
pub fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore terminal to normal mode
pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

/// Install panic hook that restores terminal
pub fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
        original_hook(panic_info);
    }));
}

/// Main TUI event loop
pub async fn run_tui(
    profile: ResolvedProfile,
    available_profiles: Vec<String>,
) -> Result<()> {
    install_panic_hook();
    let mut terminal = init_terminal()?;

    let mut app = App::new(profile, available_profiles);

    // Create channel for background task updates
    let (tx, mut rx) = mpsc::channel::<TaskUpdate>(100);

    // Start checking all packages in parallel
    start_parallel_check(&app.packages, tx.clone()).await;

    loop {
        // Render current screen
        terminal.draw(|frame| {
            match app.screen {
                Screen::Dashboard => screens::dashboard::render(frame, &app),
                Screen::Selector => screens::selector::render(frame, &app),
                Screen::Installer => screens::installer::render(frame, &app),
                Screen::ProfilePicker => screens::profile_picker::render(frame, &app),
                Screen::Doctor => screens::doctor::render(frame, &app),
                Screen::Summary => screens::summary::render(frame, &app),
            }
        })?;

        if app.should_quit {
            break;
        }

        // Handle events with timeout
        tokio::select! {
            // Handle terminal events
            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                while event::poll(Duration::from_millis(0))? {
                    if let Event::Key(key) = event::read()? {
                        handle_key_event(&mut app, key, &tx).await;
                    }
                }
            }
            // Handle background task updates
            Some(update) = rx.recv() => {
                handle_task_update(&mut app, update);
            }
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}

/// Start parallel package status checks
async fn start_parallel_check(
    packages: &[PackageState],
    tx: mpsc::Sender<TaskUpdate>,
) {
    for (index, pkg_state) in packages.iter().enumerate() {
        let tx = tx.clone();
        let package = pkg_state.package.clone();
        tokio::spawn(async move {
            let _ = tx.send(TaskUpdate::CheckStarted { index }).await;
            let status = crate::engine::check::check_package(&package)
                .await
                .unwrap_or(PackageStatus::Unknown);
            let _ = tx.send(TaskUpdate::CheckComplete { index, status }).await;
        });
    }
}

/// Start parallel installation of selected packages
async fn start_parallel_install(
    app: &mut App,
    tx: mpsc::Sender<TaskUpdate>,
) {
    let mut install_count = 0;
    for (index, pkg_state) in app.packages.iter().enumerate() {
        if !pkg_state.selected {
            continue;
        }
        match &pkg_state.status {
            PackageStatus::NotInstalled | PackageStatus::Outdated { .. } | PackageStatus::Unknown => {
                install_count += 1;
                let tx = tx.clone();
                let package = pkg_state.package.clone();
                let is_outdated = matches!(&pkg_state.status, PackageStatus::Outdated { .. });
                tokio::spawn(async move {
                    let _ = tx.send(TaskUpdate::InstallStarted { index }).await;
                    let result = if is_outdated {
                        crate::engine::install::upgrade_package(&package).await
                    } else {
                        crate::engine::install::install_package(&package).await
                    };
                    let (success, message) = match result {
                        Ok(()) => (true, "Success".to_string()),
                        Err(e) => (false, format!("{}", e)),
                    };
                    let _ = tx
                        .send(TaskUpdate::InstallComplete {
                            index,
                            success,
                            message,
                        })
                        .await;
                });
            }
            _ => {}
        }
    }
    app.install_total = install_count;
    app.screen = Screen::Installer;
}

/// Handle keyboard input
async fn handle_key_event(
    app: &mut App,
    key: crossterm::event::KeyEvent,
    tx: &mpsc::Sender<TaskUpdate>,
) {
    // Global quit
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true;
        return;
    }

    match app.screen {
        Screen::Dashboard => match key.code {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('i') => app.screen = Screen::Selector,
            KeyCode::Char('u') => {
                // Upgrade all outdated
                start_parallel_install(app, tx.clone()).await;
            }
            KeyCode::Char('s') => {
                // Sync configs
                app.summary_messages.push("Syncing configs...".to_string());
            }
            KeyCode::Char('d') => app.screen = Screen::Doctor,
            KeyCode::Up | KeyCode::Char('k') => {
                if app.selected_index > 0 {
                    app.selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if app.selected_index < app.packages.len().saturating_sub(1) {
                    app.selected_index += 1;
                }
            }
            _ => {}
        },
        Screen::Selector => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => app.screen = Screen::Dashboard,
            KeyCode::Up | KeyCode::Char('k') => {
                if app.selected_index > 0 {
                    app.selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if app.selected_index < app.packages.len().saturating_sub(1) {
                    app.selected_index += 1;
                }
            }
            KeyCode::Char(' ') => {
                if let Some(pkg) = app.packages.get_mut(app.selected_index) {
                    pkg.selected = !pkg.selected;
                }
            }
            KeyCode::Char('a') => {
                for pkg in &mut app.packages {
                    pkg.selected = true;
                }
            }
            KeyCode::Char('n') => {
                for pkg in &mut app.packages {
                    pkg.selected = false;
                }
            }
            KeyCode::Enter => {
                start_parallel_install(app, tx.clone()).await;
            }
            _ => {}
        },
        Screen::Installer => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if app.install_complete + app.install_failed >= app.install_total {
                    app.screen = Screen::Summary;
                }
            }
            _ => {}
        },
        Screen::ProfilePicker => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => app.screen = Screen::Dashboard,
            KeyCode::Up | KeyCode::Char('k') => {
                if app.profile_selected_index > 0 {
                    app.profile_selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if app.profile_selected_index < app.available_profiles.len().saturating_sub(1) {
                    app.profile_selected_index += 1;
                }
            }
            _ => {}
        },
        Screen::Doctor => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => app.screen = Screen::Dashboard,
            _ => {}
        },
        Screen::Summary => match key.code {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter => {
                app.screen = Screen::Dashboard;
            }
            _ => {}
        },
    }
}

/// Handle updates from background tasks
fn handle_task_update(app: &mut App, update: TaskUpdate) {
    match update {
        TaskUpdate::CheckStarted { index } => {
            if let Some(pkg) = app.packages.get_mut(index) {
                pkg.task_status = TaskStatus::Checking;
            }
        }
        TaskUpdate::CheckComplete { index, status } => {
            if let Some(pkg) = app.packages.get_mut(index) {
                pkg.status = status;
                pkg.task_status = TaskStatus::Idle;
            }
        }
        TaskUpdate::InstallStarted { index } => {
            if let Some(pkg) = app.packages.get_mut(index) {
                match &pkg.status {
                    PackageStatus::Outdated { .. } => pkg.task_status = TaskStatus::Upgrading,
                    _ => pkg.task_status = TaskStatus::Installing,
                }
            }
        }
        TaskUpdate::InstallComplete {
            index,
            success,
            message,
        } => {
            if let Some(pkg) = app.packages.get_mut(index) {
                if success {
                    pkg.task_status = TaskStatus::Done;
                    app.install_complete += 1;
                    app.summary_messages
                        .push(format!("Installed: {}", pkg.package.id));
                } else {
                    pkg.task_status = TaskStatus::Failed(message.clone());
                    app.install_failed += 1;
                    app.summary_messages
                        .push(format!("Failed: {} - {}", pkg.package.id, message));
                }
            }
        }
    }
}
