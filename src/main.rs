mod cli;
mod config;
mod engine;
mod providers;
mod system;
mod tui;

use clap::Parser;
use color_eyre::Result;

use crate::cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

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
                println!("Export not yet implemented");
            }
        },
        None => {
            // Default: launch TUI
            tracing::info!("Launching TUI");
            engine::run_tui_mode().await?;
        }
    }

    Ok(())
}
