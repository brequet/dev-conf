use color_eyre::Result;

use crate::config::schema::{Package, PackageSource};
use crate::providers;
use crate::system::shell;

/// Install a single package, running before/after hooks
pub async fn install_package(package: &Package) -> Result<()> {
    // Run before hook
    if let Some(before) = &package.before {
        tracing::info!("Running before hook for {}", package.id);
        let output = shell::run_powershell(before).await?;
        if !output.success {
            tracing::error!("Before hook failed for {}: {}", package.id, output.stderr);
            return Err(color_eyre::eyre::eyre!(
                "Before hook failed for {}: {}",
                package.id,
                output.stderr
            ));
        }
    }

    // Install
    let result = match &package.source {
        PackageSource::Winget => providers::winget::install(package).await,
        PackageSource::GitHub(gh) => providers::github::install(package, gh).await,
        PackageSource::Scoop => providers::scoop::install(package).await,
        PackageSource::Choco => providers::choco::install(package).await,
    };

    result?;

    // Run after hook
    if let Some(after) = &package.after {
        tracing::info!("Running after hook for {}", package.id);
        let output = shell::run_powershell(after).await?;
        if !output.success {
            tracing::warn!("After hook failed for {}: {}", package.id, output.stderr);
        }
    }

    Ok(())
}

/// Upgrade a single package
pub async fn upgrade_package(package: &Package) -> Result<()> {
    match &package.source {
        PackageSource::Winget => providers::winget::upgrade(package).await,
        PackageSource::GitHub(gh) => providers::github::install(package, gh).await,
        PackageSource::Scoop => providers::scoop::upgrade(package).await,
        PackageSource::Choco => providers::choco::upgrade(package).await,
    }
}
