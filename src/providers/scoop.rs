use color_eyre::Result;

use crate::config::schema::{Package, PackageStatus};
use crate::system::shell;

/// Check if a scoop package is installed
pub async fn check(package: &Package) -> Result<PackageStatus> {
    let cmd = format!("scoop info {}", package.id);
    let output = shell::run_powershell(&cmd).await?;

    if !output.success {
        return Ok(PackageStatus::NotInstalled);
    }

    // Parse scoop info output for version
    for line in output.stdout.lines() {
        if line.trim().starts_with("Version") || line.trim().starts_with("Installed") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                let version = parts[1].trim().to_string();
                return Ok(PackageStatus::Installed { version });
            }
        }
    }

    Ok(PackageStatus::Installed {
        version: "unknown".to_string(),
    })
}

/// Install a scoop package
pub async fn install(package: &Package) -> Result<()> {
    let cmd = format!("scoop install {}", package.id);
    let output = shell::run_powershell(&cmd).await?;

    if !output.success {
        return Err(color_eyre::eyre::eyre!(
            "scoop install failed for {}: {}",
            package.id,
            output.stderr
        ));
    }

    println!("    Installed {} via scoop", package.id);
    Ok(())
}

/// Upgrade a scoop package
pub async fn upgrade(package: &Package) -> Result<()> {
    let cmd = format!("scoop update {}", package.id);
    let output = shell::run_powershell(&cmd).await?;

    if !output.success {
        return Err(color_eyre::eyre::eyre!(
            "scoop upgrade failed for {}: {}",
            package.id,
            output.stderr
        ));
    }

    println!("    Upgraded {} via scoop", package.id);
    Ok(())
}
