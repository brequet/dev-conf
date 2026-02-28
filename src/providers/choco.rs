use color_eyre::Result;

use crate::config::schema::{Package, PackageStatus};
use crate::system::shell;

/// Check if a chocolatey package is installed
pub async fn check(package: &Package) -> Result<PackageStatus> {
    let cmd = format!("choco list {} --exact --local-only", package.id);
    let output = shell::run_powershell(&cmd).await?;

    if !output.success {
        return Ok(PackageStatus::NotInstalled);
    }

    // Parse choco list output
    for line in output.stdout.lines() {
        if line.contains(&package.id) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return Ok(PackageStatus::Installed {
                    version: parts[1].to_string(),
                });
            }
        }
    }

    Ok(PackageStatus::NotInstalled)
}

/// Install a chocolatey package
pub async fn install(package: &Package) -> Result<()> {
    let cmd = format!("choco install {} -y", package.id);
    let output = shell::run_powershell(&cmd).await?;

    if !output.success {
        return Err(color_eyre::eyre::eyre!(
            "choco install failed for {}: {}",
            package.id,
            output.stderr
        ));
    }

    println!("    Installed {} via chocolatey", package.id);
    Ok(())
}

/// Upgrade a chocolatey package
pub async fn upgrade(package: &Package) -> Result<()> {
    let cmd = format!("choco upgrade {} -y", package.id);
    let output = shell::run_powershell(&cmd).await?;

    if !output.success {
        return Err(color_eyre::eyre::eyre!(
            "choco upgrade failed for {}: {}",
            package.id,
            output.stderr
        ));
    }

    println!("    Upgraded {} via chocolatey", package.id);
    Ok(())
}
