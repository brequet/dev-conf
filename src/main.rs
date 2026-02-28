mod cli;
mod config;
mod context;
mod engine;
mod providers;
mod system;
mod tui;

use clap::Parser;
use color_eyre::Result;

use crate::cli::{Cli, Command};
use crate::context::RunContext;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    // Initialize global runtime context from CLI flags
    context::init(RunContext {
        dry_run: cli.dry_run,
        no_tui: cli.no_tui,
        verbose: cli.verbose,
        parallel: cli.parallel,
        max_retries: 2,
    });

    // Set up tracing: file appender (TUI owns stdout, logs go to file)
    let log_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("devconf")
        .join("logs");
    std::fs::create_dir_all(&log_dir)?;

    let file_appender = tracing_appender::rolling::daily(&log_dir, "devconf.log");

    let log_level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(log_level.into()),
        )
        .with_ansi(false)
        .init();

    if cli.dry_run {
        println!("[dry-run] Dry-run mode enabled -- no changes will be made");
    }

    match cli.command {
        Some(cmd) => match cmd {
            Command::Install(args) => {
                tracing::info!("Install command invoked");
                engine::run_install(args).await?;
            }
            Command::Status(args) => {
                tracing::info!("Status command invoked");
                engine::run_status(args).await?;
            }
            Command::Sync(args) => {
                tracing::info!("Sync command invoked");
                engine::run_sync(args).await?;
            }
            Command::Upgrade(args) => {
                tracing::info!("Upgrade command invoked");
                engine::run_upgrade(args).await?;
            }
            Command::Doctor(args) => {
                tracing::info!("Doctor command invoked");
                engine::run_doctor(args).await?;
            }
            Command::Profile(args) => {
                tracing::info!("Profile command invoked");
                engine::run_profile(args).await?;
            }
            Command::Export => {
                tracing::info!("Export command invoked");
                engine::export::run_export().await?;
            }
            Command::Completions { shell } => {
                tracing::info!("Completions command invoked for {:?}", shell);
                let mut cmd = <Cli as clap::CommandFactory>::command();
                clap_complete::generate(shell, &mut cmd, "devconf", &mut std::io::stdout());
            }
        },
        None => {
            // Default: launch TUI (unless --no-tui)
            if cli.no_tui {
                tracing::info!("Running default status in plain text mode (--no-tui)");
                engine::run_status_plain().await?;
            } else {
                tracing::info!("Launching TUI");
                engine::run_tui_mode().await?;
            }
        }
    }

    Ok(())
}
