use color_eyre::Result;

use crate::config::schema::{Package, PackageStatus};
use crate::system::shell;

/// Check if a winget package is installed
pub async fn check(package: &Package) -> Result<PackageStatus> {
    let cmd = format!("winget list --id {} --exact --accept-source-agreements", package.id);
    let output = shell::run_powershell(&cmd).await?;

    if !output.success {
        return Ok(PackageStatus::NotInstalled);
    }

    let stdout = &output.stdout;

    // Parse winget list output to determine version info
    // winget list output has lines like: "PackageName Id Version Available"
    for line in stdout.lines() {
        if line.contains(&package.id) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // Try to find version and available columns
            if parts.len() >= 4 {
                let version = parts[parts.len() - 2].to_string();
                let available = parts[parts.len() - 1].to_string();
                // If the last part looks like a version, it might be an upgrade
                if available.contains('.') && version.contains('.') && available != version {
                    return Ok(PackageStatus::Outdated {
                        current: version,
                        available,
                    });
                }
            }
            // Extract version (usually second-to-last or last column)
            if let Some(version) = parts.last() {
                return Ok(PackageStatus::Installed {
                    version: version.to_string(),
                });
            }
        }
    }

    // If winget list succeeded but we couldn't parse, it's installed
    Ok(PackageStatus::Installed {
        version: "unknown".to_string(),
    })
}

/// Install a winget package
pub async fn install(package: &Package) -> Result<()> {
    let cmd = format!(
        "winget install --id {} --exact --accept-source-agreements --accept-package-agreements",
        package.id
    );
    let output = shell::run_powershell(&cmd).await?;

    if !output.success {
        return Err(color_eyre::eyre::eyre!(
            "winget install failed for {}: {}",
            package.id,
            output.stderr
        ));
    }

    println!("    Installed {} via winget", package.id);
    Ok(())
}

/// Upgrade a winget package
pub async fn upgrade(package: &Package) -> Result<()> {
    let cmd = format!(
        "winget upgrade --id {} --exact --accept-source-agreements --accept-package-agreements",
        package.id
    );
    let output = shell::run_powershell(&cmd).await?;

    if !output.success {
        return Err(color_eyre::eyre::eyre!(
            "winget upgrade failed for {}: {}",
            package.id,
            output.stderr
        ));
    }

    println!("    Upgraded {} via winget", package.id);
    Ok(())
}
