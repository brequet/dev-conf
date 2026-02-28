use color_eyre::Result;

use super::shell;

/// Check if a directory is in the user PATH
pub async fn is_in_user_path(dir: &str) -> Result<bool> {
    let cmd = format!(
        "$path = [Environment]::GetEnvironmentVariable('PATH', 'User'); $path -split ';' -contains '{}'",
        dir
    );
    let output = shell::run_powershell(&cmd).await?;

    Ok(output.success && output.stdout.trim().eq_ignore_ascii_case("true"))
}

/// Add a directory to the user PATH (idempotent)
pub async fn add_to_user_path(dir: &str) -> Result<()> {
    if is_in_user_path(dir).await? {
        println!("  PATH already contains {}", dir);
        return Ok(());
    }

    let cmd = format!(
        "$current = [Environment]::GetEnvironmentVariable('PATH', 'User'); \
         [Environment]::SetEnvironmentVariable('PATH', \"$current;{}\", 'User')",
        dir
    );
    let output = shell::run_powershell(&cmd).await?;

    if !output.success {
        return Err(color_eyre::eyre::eyre!(
            "Failed to add {} to PATH: {}",
            dir,
            output.stderr
        ));
    }

    println!("  Added {} to PATH", dir);
    Ok(())
}
