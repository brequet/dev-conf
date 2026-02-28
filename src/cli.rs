use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "devconf",
    version,
    about = "Manage development environments across Windows machines"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Show what would happen without doing anything
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Disable TUI, use plain text output
    #[arg(long, global = true)]
    pub no_tui: bool,

    /// Maximum concurrent operations
    #[arg(long, global = true, default_value = "4")]
    pub parallel: usize,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Install packages for a profile
    Install(InstallArgs),
    /// Show what's installed, outdated, or missing
    Status(StatusArgs),
    /// Deploy config files, env vars, PATH only (no apps)
    Sync(SyncArgs),
    /// Upgrade all outdated packages
    Upgrade(UpgradeArgs),
    /// Verify system health
    Doctor(DoctorArgs),
    /// Manage profiles
    Profile(ProfileArgs),
    /// Scan current machine and generate a profile YAML
    Export,
}

#[derive(clap::Args, Debug)]
pub struct InstallArgs {
    /// Install everything non-interactively
    #[arg(long)]
    pub all: bool,

    /// Use a specific profile
    #[arg(long)]
    pub profile: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct StatusArgs {
    /// Use a specific profile
    #[arg(long)]
    pub profile: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct SyncArgs {
    /// Use a specific profile
    #[arg(long)]
    pub profile: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct UpgradeArgs {
    /// Use a specific profile
    #[arg(long)]
    pub profile: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct DoctorArgs {
    /// Use a specific profile
    #[arg(long)]
    pub profile: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct ProfileArgs {
    #[command(subcommand)]
    pub command: ProfileCommand,
}

#[derive(Subcommand, Debug)]
pub enum ProfileCommand {
    /// List available profiles
    List,
    /// Set the active profile
    Set {
        /// Profile name to activate
        name: String,
    },
}
