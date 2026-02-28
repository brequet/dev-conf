pub mod check;
pub mod config_deploy;
pub mod install;

use color_eyre::{Result, eyre::eyre};

use crate::cli::{
    DoctorArgs, InstallArgs, ProfileArgs, ProfileCommand, StatusArgs, SyncArgs, UpgradeArgs,
};
use crate::config::schema::{
    Package, PackageEntry, PackageSource, ProfileFile, ResolvedProfile, RootConfig,
};
use crate::config::vars;

/// Load root config from devconf.yaml
fn load_root_config() -> Result<RootConfig> {
    let path = std::path::Path::new("devconf.yaml");
    if !path.exists() {
        return Ok(RootConfig {
            schema: 1,
            vars: std::collections::HashMap::new(),
        });
    }
    let content = std::fs::read_to_string(path)?;
    let config: RootConfig = serde_yaml::from_str(&content)?;
    Ok(config)
}

/// Load a profile file by name
fn load_profile(name: &str) -> Result<ProfileFile> {
    let path = format!("profiles/{}.yaml", name);
    let content = std::fs::read_to_string(&path)
        .map_err(|e| eyre!("Failed to load profile '{}' from {}: {}", name, path, e))?;
    let profile: ProfileFile = serde_yaml::from_str(&content)?;
    Ok(profile)
}

/// Normalize a PackageEntry into a Package (or a remove directive)
fn normalize_package(entry: &PackageEntry) -> Result<NormalizedEntry> {
    match entry {
        PackageEntry::Tagged(tagged) => {
            if let Some(id) = &tagged.winget {
                Ok(NormalizedEntry::Package(Package {
                    source: PackageSource::Winget,
                    id: id.clone(),
                    before: tagged.before.clone(),
                    after: tagged.after.clone(),
                }))
            } else if let Some(id) = &tagged.scoop {
                Ok(NormalizedEntry::Package(Package {
                    source: PackageSource::Scoop,
                    id: id.clone(),
                    before: tagged.before.clone(),
                    after: tagged.after.clone(),
                }))
            } else if let Some(id) = &tagged.choco {
                Ok(NormalizedEntry::Package(Package {
                    source: PackageSource::Choco,
                    id: id.clone(),
                    before: tagged.before.clone(),
                    after: tagged.after.clone(),
                }))
            } else if let Some(gh) = &tagged.github {
                Ok(NormalizedEntry::Package(Package {
                    source: PackageSource::GitHub(gh.clone()),
                    id: gh.repo.clone(),
                    before: tagged.before.clone(),
                    after: tagged.after.clone(),
                }))
            } else if let Some(id) = &tagged.remove {
                Ok(NormalizedEntry::Remove(id.clone()))
            } else {
                Err(eyre!("Package entry has no recognized source"))
            }
        }
    }
}

enum NormalizedEntry {
    Package(Package),
    Remove(String),
}

/// Resolve a profile by loading it and merging with its parent (extends)
pub fn resolve_profile(name: &str, root_config: &RootConfig) -> Result<ResolvedProfile> {
    let profile = load_profile(name)?;

    // Build variable map
    let mut var_map = vars::build_default_vars()?;
    for (k, v) in &root_config.vars {
        var_map.insert(k.clone(), v.clone());
    }

    // If extends, load parent first
    let mut packages: Vec<Package> = Vec::new();
    let mut configs = Vec::new();
    let mut env = std::collections::HashMap::new();
    let mut path = Vec::new();
    let mut actions = Vec::new();

    if let Some(parent_name) = &profile.extends {
        let parent = resolve_profile(parent_name, root_config)?;
        packages = parent.packages;
        configs = parent.configs;
        env = parent.env;
        path = parent.path;
        actions = parent.actions;
    }

    // Process child packages: add new ones, remove marked ones
    let mut removals: Vec<String> = Vec::new();
    for entry in &profile.packages {
        match normalize_package(entry)? {
            NormalizedEntry::Package(pkg) => packages.push(pkg),
            NormalizedEntry::Remove(id) => removals.push(id),
        }
    }

    // Apply removals
    packages.retain(|p| !removals.contains(&p.id));

    // Merge configs
    configs.extend(profile.configs.clone());

    // Merge env (child overrides parent)
    for (k, v) in &profile.env {
        env.insert(k.clone(), v.clone());
    }

    // Merge path
    path.extend(profile.path.clone());

    // Merge actions
    actions.extend(profile.actions.clone());

    // Resolve template variables in all string fields
    let resolved_packages: Vec<Package> = packages
        .into_iter()
        .map(|mut p| {
            p.id = vars::resolve_vars(&p.id, &var_map);
            if let PackageSource::GitHub(ref mut gh) = p.source {
                gh.repo = vars::resolve_vars(&gh.repo, &var_map);
                gh.asset = vars::resolve_vars(&gh.asset, &var_map);
                if let Some(ref mut r) = gh.rename {
                    *r = vars::resolve_vars(r, &var_map);
                }
                if let Some(ref mut t) = gh.to {
                    *t = vars::resolve_vars(t, &var_map);
                }
            }
            p
        })
        .collect();

    let resolved_configs = configs
        .into_iter()
        .map(|mut c| {
            c.src = vars::resolve_vars(&c.src, &var_map);
            c.dest = vars::resolve_vars(&c.dest, &var_map);
            c
        })
        .collect();

    let resolved_env = env
        .into_iter()
        .map(|(k, v)| (k, vars::resolve_vars(&v, &var_map)))
        .collect();

    let resolved_path = path
        .into_iter()
        .map(|p| vars::resolve_vars(&p, &var_map))
        .collect();

    let resolved_actions = actions
        .into_iter()
        .map(|mut a| {
            a.shell = vars::resolve_vars(&a.shell, &var_map);
            a
        })
        .collect();

    Ok(ResolvedProfile {
        name: profile.profile,
        packages: resolved_packages,
        configs: resolved_configs,
        env: resolved_env,
        path: resolved_path,
        actions: resolved_actions,
    })
}

/// Determine which profile to use
fn determine_profile(profile_arg: &Option<String>) -> Result<String> {
    if let Some(name) = profile_arg {
        return Ok(name.clone());
    }

    // Try to read from state file
    let state_path = dirs::config_dir()
        .ok_or_else(|| eyre!("Cannot determine config directory"))?
        .join("devconf")
        .join("state.json");

    if state_path.exists() {
        let content = std::fs::read_to_string(&state_path)?;
        let state: serde_json::Value = serde_json::from_str(&content)?;
        if let Some(profile) = state.get("active_profile").and_then(|v| v.as_str()) {
            return Ok(profile.to_string());
        }
    }

    Err(eyre!(
        "No profile specified. Use --profile <name> or set an active profile with `devconf profile set <name>`"
    ))
}

/// List available profiles by scanning the profiles/ directory
fn list_profiles() -> Result<Vec<String>> {
    let profiles_dir = std::path::Path::new("profiles");
    if !profiles_dir.exists() {
        return Ok(Vec::new());
    }

    let mut profiles = Vec::new();
    for entry in std::fs::read_dir(profiles_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                profiles.push(stem.to_string());
            }
        }
    }
    profiles.sort();
    Ok(profiles)
}

/// Save active profile to state file
fn save_active_profile(name: &str) -> Result<()> {
    let state_dir = dirs::config_dir()
        .ok_or_else(|| eyre!("Cannot determine config directory"))?
        .join("devconf");

    std::fs::create_dir_all(&state_dir)?;
    let state_path = state_dir.join("state.json");

    let mut state = if state_path.exists() {
        let content = std::fs::read_to_string(&state_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    state["active_profile"] = serde_json::json!(name);
    std::fs::write(&state_path, serde_json::to_string_pretty(&state)?)?;
    Ok(())
}

// === Command handlers ===

pub async fn run_install(args: InstallArgs) -> Result<()> {
    let profile_name = determine_profile(&args.profile)?;
    let root_config = load_root_config()?;
    let profile = resolve_profile(&profile_name, &root_config)?;

    println!("Installing profile: {}", profile.name);
    println!("Packages: {}", profile.packages.len());

    for pkg in &profile.packages {
        let status = check::check_package(pkg).await?;
        match status {
            crate::config::schema::PackageStatus::NotInstalled => {
                println!("  Installing {}...", pkg.id);
                install::install_package(pkg).await?;
            }
            crate::config::schema::PackageStatus::Outdated { current, available } => {
                println!("  Upgrading {} ({} -> {})...", pkg.id, current, available);
                install::upgrade_package(pkg).await?;
            }
            crate::config::schema::PackageStatus::Installed { version } => {
                println!("  {} already installed ({})", pkg.id, version);
            }
            crate::config::schema::PackageStatus::Unknown => {
                println!("  {} status unknown, attempting install...", pkg.id);
                install::install_package(pkg).await?;
            }
        }
    }

    // Deploy configs
    config_deploy::deploy_configs(&profile).await?;

    println!("\nInstallation complete.");
    Ok(())
}

pub async fn run_status(args: StatusArgs) -> Result<()> {
    let profile_name = determine_profile(&args.profile)?;
    let root_config = load_root_config()?;
    let profile = resolve_profile(&profile_name, &root_config)?;

    println!("Profile: {}", profile.name);
    println!("{:-<60}", "");

    for pkg in &profile.packages {
        let source_name = match &pkg.source {
            PackageSource::Winget => "winget",
            PackageSource::Scoop => "scoop",
            PackageSource::Choco => "choco",
            PackageSource::GitHub(_) => "github",
        };
        let status = check::check_package(pkg).await?;
        let status_symbol = match &status {
            crate::config::schema::PackageStatus::Installed { .. } => "OK",
            crate::config::schema::PackageStatus::Outdated { .. } => "UP",
            crate::config::schema::PackageStatus::NotInstalled => "XX",
            crate::config::schema::PackageStatus::Unknown => "??",
        };
        println!("  [{}] {:8} {:<30} {}", status_symbol, source_name, pkg.id, status);
    }

    Ok(())
}

pub async fn run_sync(args: SyncArgs) -> Result<()> {
    let profile_name = determine_profile(&args.profile)?;
    let root_config = load_root_config()?;
    let profile = resolve_profile(&profile_name, &root_config)?;

    println!("Syncing configs for profile: {}", profile.name);
    config_deploy::deploy_configs(&profile).await?;
    println!("Sync complete.");
    Ok(())
}

pub async fn run_upgrade(args: UpgradeArgs) -> Result<()> {
    let profile_name = determine_profile(&args.profile)?;
    let root_config = load_root_config()?;
    let profile = resolve_profile(&profile_name, &root_config)?;

    println!("Checking for upgrades in profile: {}", profile.name);

    for pkg in &profile.packages {
        let status = check::check_package(pkg).await?;
        if let crate::config::schema::PackageStatus::Outdated { current, available } = status {
            println!("  Upgrading {} ({} -> {})...", pkg.id, current, available);
            install::upgrade_package(pkg).await?;
        }
    }

    println!("Upgrade complete.");
    Ok(())
}

pub async fn run_doctor(args: DoctorArgs) -> Result<()> {
    let profile_name = determine_profile(&args.profile)?;
    let root_config = load_root_config()?;
    let profile = resolve_profile(&profile_name, &root_config)?;

    println!("Doctor check for profile: {}", profile.name);
    println!("{:-<60}", "");

    let mut issues = 0;

    // Check packages
    for pkg in &profile.packages {
        let status = check::check_package(pkg).await?;
        match &status {
            crate::config::schema::PackageStatus::NotInstalled => {
                println!("  [FAIL] Package {} is not installed", pkg.id);
                issues += 1;
            }
            crate::config::schema::PackageStatus::Outdated { current, available } => {
                println!(
                    "  [WARN] Package {} is outdated ({} -> {})",
                    pkg.id, current, available
                );
                issues += 1;
            }
            crate::config::schema::PackageStatus::Installed { .. } => {
                println!("  [ OK ] Package {}", pkg.id);
            }
            crate::config::schema::PackageStatus::Unknown => {
                println!("  [WARN] Package {} status unknown", pkg.id);
                issues += 1;
            }
        }
    }

    // Check configs
    for cfg in &profile.configs {
        let dest = std::path::Path::new(&cfg.dest);
        if dest.exists() {
            println!("  [ OK ] Config {}", cfg.dest);
        } else {
            println!("  [FAIL] Config {} is missing", cfg.dest);
            issues += 1;
        }
    }

    // Check env vars
    for (key, expected) in &profile.env {
        match std::env::var(key) {
            Ok(val) if val == *expected => {
                println!("  [ OK ] Env {} = {}", key, expected);
            }
            Ok(val) => {
                println!("  [WARN] Env {} = {} (expected {})", key, val, expected);
                issues += 1;
            }
            Err(_) => {
                println!("  [FAIL] Env {} is not set", key);
                issues += 1;
            }
        }
    }

    // Check PATH entries
    let current_path = std::env::var("PATH").unwrap_or_default();
    for p in &profile.path {
        if current_path.contains(p) {
            println!("  [ OK ] PATH contains {}", p);
        } else {
            println!("  [FAIL] PATH missing {}", p);
            issues += 1;
        }
    }

    println!("{:-<60}", "");
    if issues == 0 {
        println!("All checks passed.");
    } else {
        println!("{} issue(s) found.", issues);
    }

    Ok(())
}

pub async fn run_profile(args: ProfileArgs) -> Result<()> {
    match args.command {
        ProfileCommand::List => {
            let profiles = list_profiles()?;
            if profiles.is_empty() {
                println!("No profiles found in profiles/ directory.");
            } else {
                println!("Available profiles:");
                for p in &profiles {
                    println!("  - {}", p);
                }
            }
        }
        ProfileCommand::Set { name } => {
            // Verify the profile exists
            let _profile = load_profile(&name)?;
            save_active_profile(&name)?;
            println!("Active profile set to: {}", name);
        }
    }
    Ok(())
}
