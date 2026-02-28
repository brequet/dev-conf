use color_eyre::Result;
use std::path::Path;

use crate::config::schema::{ConfigMethod, ResolvedProfile};
use crate::context;

/// Deploy all config files for a profile
pub async fn deploy_configs(profile: &ResolvedProfile) -> Result<()> {
    for cfg in &profile.configs {
        deploy_single_config(cfg).await?;
    }

    // Set environment variables
    for (key, value) in &profile.env {
        set_env_var(key, value).await?;
    }

    // Add PATH entries
    for path_entry in &profile.path {
        add_to_path(path_entry).await?;
    }

    Ok(())
}

async fn deploy_single_config(cfg: &crate::config::schema::ConfigEntry) -> Result<()> {
    let src = Path::new(&cfg.src);
    let dest = Path::new(&cfg.dest);

    if context::is_dry_run() {
        let method = match cfg.method {
            ConfigMethod::Symlink => "symlink",
            ConfigMethod::Copy => "copy",
        };
        println!(
            "  [dry-run] Would {} {} -> {}",
            method,
            src.display(),
            dest.display()
        );
        return Ok(());
    }

    // Create parent directories if mkdir is set
    if cfg.mkdir {
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    match cfg.method {
        ConfigMethod::Symlink => deploy_symlink(src, dest)?,
        ConfigMethod::Copy => deploy_copy(src, dest)?,
    }

    Ok(())
}

fn deploy_symlink(src: &Path, dest: &Path) -> Result<()> {
    // Check if already a correct symlink
    if dest.is_symlink() {
        let target = std::fs::read_link(dest)?;
        let abs_src = std::fs::canonicalize(src)?;
        if target == abs_src {
            println!("  Symlink OK: {}", dest.display());
            return Ok(());
        }
        // Wrong target, remove and recreate
        println!("  Symlink target changed, recreating: {}", dest.display());
        std::fs::remove_file(dest)?;
    } else if dest.exists() {
        // Backup existing file
        let backup = dest.with_extension("bak");
        println!(
            "  Backing up existing file: {} -> {}",
            dest.display(),
            backup.display()
        );
        std::fs::rename(dest, &backup)?;
    }

    // Create parent dirs if they don't exist
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let abs_src = std::fs::canonicalize(src)?;
    println!(
        "  Creating symlink: {} -> {}",
        dest.display(),
        abs_src.display()
    );

    #[cfg(windows)]
    {
        if src.is_dir() {
            std::os::windows::fs::symlink_dir(&abs_src, dest)?;
        } else {
            std::os::windows::fs::symlink_file(&abs_src, dest)?;
        }
    }

    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(&abs_src, dest)?;
    }

    Ok(())
}

fn deploy_copy(src: &Path, dest: &Path) -> Result<()> {
    // If source is a directory, copy recursively
    if src.is_dir() {
        return copy_dir_recursive(src, dest);
    }

    // Compare hashes if dest exists
    if dest.exists() {
        let src_hash = file_hash(src)?;
        let dest_hash = file_hash(dest)?;
        if src_hash == dest_hash {
            println!("  Copy OK (identical): {}", dest.display());
            return Ok(());
        }
        // Backup and copy
        let backup = dest.with_extension("bak");
        println!(
            "  Backing up: {} -> {}",
            dest.display(),
            backup.display()
        );
        std::fs::rename(dest, &backup)?;
    }

    // Create parent dirs
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    println!("  Copying: {} -> {}", src.display(), dest.display());
    std::fs::copy(src, dest)?;
    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            deploy_copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}

fn file_hash(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let data = std::fs::read(path)?;
    let hash = Sha256::digest(&data);
    Ok(format!("{:x}", hash))
}

async fn set_env_var(key: &str, value: &str) -> Result<()> {
    if context::is_dry_run() {
        println!("  [dry-run] Would set env {} = {}", key, value);
        return Ok(());
    }
    crate::system::env::set_user_env_var(key, value).await
}

async fn add_to_path(entry: &str) -> Result<()> {
    if context::is_dry_run() {
        println!("  [dry-run] Would add {} to PATH", entry);
        return Ok(());
    }
    crate::system::path::add_to_user_path(entry).await
}
