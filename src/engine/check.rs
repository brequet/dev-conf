use color_eyre::Result;

use crate::config::schema::{Package, PackageSource, PackageStatus};
use crate::providers;

/// Check the status of a single package
pub async fn check_package(package: &Package) -> Result<PackageStatus> {
    match &package.source {
        PackageSource::Winget => providers::winget::check(package).await,
        PackageSource::GitHub(gh) => providers::github::check(package, gh).await,
        PackageSource::Scoop => providers::scoop::check(package).await,
        PackageSource::Choco => providers::choco::check(package).await,
    }
}
