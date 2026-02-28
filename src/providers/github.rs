use color_eyre::{Result, eyre::eyre};
use std::path::PathBuf;

use crate::config::schema::{GitHubPackage, Package, PackageStatus};

/// Check if a GitHub release binary is installed
pub async fn check(_package: &Package, gh: &GitHubPackage) -> Result<PackageStatus> {
    let target_path = get_target_path(gh);

    if !target_path.exists() {
        return Ok(PackageStatus::NotInstalled);
    }

    // Check state file for version info
    let state = load_github_state()?;
    if let Some(version) = state.get(&gh.repo) {
        // Check latest release
        match get_latest_release_tag(&gh.repo).await {
            Ok(latest) => {
                if version.as_str() == Some(&latest) {
                    return Ok(PackageStatus::Installed {
                        version: latest,
                    });
                } else {
                    return Ok(PackageStatus::Outdated {
                        current: version.as_str().unwrap_or("unknown").to_string(),
                        available: latest,
                    });
                }
            }
            Err(e) => {
                tracing::warn!("Failed to check latest release for {}: {}", gh.repo, e);
                return Ok(PackageStatus::Installed {
                    version: version.as_str().unwrap_or("unknown").to_string(),
                });
            }
        }
    }

    Ok(PackageStatus::Installed {
        version: "unknown".to_string(),
    })
}

/// Install/download a GitHub release binary
pub async fn install(package: &Package, gh: &GitHubPackage) -> Result<()> {
    let (download_url, tag) = get_release_asset_url(&gh.repo, &gh.asset).await?;
    let target_path = get_target_path(gh);

    // Create target directory
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Download
    println!("    Downloading {} from {}...", package.id, download_url);
    let client = reqwest::Client::new();
    let response = client
        .get(&download_url)
        .header("User-Agent", "devconf")
        .header("Accept", "application/octet-stream")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(eyre!(
            "Failed to download {}: HTTP {}",
            download_url,
            response.status()
        ));
    }

    let bytes = response.bytes().await?;
    std::fs::write(&target_path, &bytes)?;

    // Save version to state
    save_github_state(&gh.repo, &tag)?;

    println!("    Installed {} ({})", package.id, tag);
    Ok(())
}

fn get_target_path(gh: &GitHubPackage) -> PathBuf {
    let filename = gh.rename.as_deref().unwrap_or(&gh.asset);
    let dir = gh
        .to
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".config").join("bin"));
    dir.join(filename)
}

async fn get_latest_release_tag(repo: &str) -> Result<String> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "devconf")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(eyre!("GitHub API error: HTTP {}", response.status()));
    }

    let json: serde_json::Value = response.json().await?;
    json["tag_name"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| eyre!("No tag_name in release response"))
}

async fn get_release_asset_url(repo: &str, asset_pattern: &str) -> Result<(String, String)> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "devconf")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(eyre!("GitHub API error: HTTP {}", response.status()));
    }

    let json: serde_json::Value = response.json().await?;
    let tag = json["tag_name"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    let assets = json["assets"]
        .as_array()
        .ok_or_else(|| eyre!("No assets in release"))?;

    for asset in assets {
        let name = asset["name"].as_str().unwrap_or("");
        if name.contains(asset_pattern) || name == asset_pattern {
            let url = asset["browser_download_url"]
                .as_str()
                .ok_or_else(|| eyre!("No download URL for asset"))?
                .to_string();
            return Ok((url, tag));
        }
    }

    Err(eyre!(
        "Asset matching '{}' not found in release for {}",
        asset_pattern,
        repo
    ))
}

fn state_file_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("devconf")
        .join("state.json")
}

fn load_github_state() -> Result<serde_json::Value> {
    let path = state_file_path();
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let content = std::fs::read_to_string(&path)?;
    let state: serde_json::Value = serde_json::from_str(&content)?;
    Ok(state.get("github_versions").cloned().unwrap_or(serde_json::json!({})))
}

fn save_github_state(repo: &str, tag: &str) -> Result<()> {
    let path = state_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut state = if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if state.get("github_versions").is_none() {
        state["github_versions"] = serde_json::json!({});
    }
    state["github_versions"][repo] = serde_json::json!(tag);

    std::fs::write(&path, serde_json::to_string_pretty(&state)?)?;
    Ok(())
}
