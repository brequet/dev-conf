use color_eyre::Result;

use super::shell;

/// Check the current value of a user environment variable
pub async fn get_user_env_var(name: &str) -> Result<Option<String>> {
    let cmd = format!(
        "[Environment]::GetEnvironmentVariable('{}', 'User')",
        name
    );
    let output = shell::run_powershell(&cmd).await?;

    if output.success {
        let value = output.stdout.trim().to_string();
        if value.is_empty() {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    } else {
        Ok(None)
    }
}

/// Set a user environment variable (idempotent)
pub async fn set_user_env_var(name: &str, value: &str) -> Result<()> {
    // Check current value first
    let current = get_user_env_var(name).await?;
    if current.as_deref() == Some(value) {
        println!("  Env {} already set correctly", name);
        return Ok(());
    }

    let cmd = format!(
        "[Environment]::SetEnvironmentVariable('{}', '{}', 'User')",
        name, value
    );
    let output = shell::run_powershell(&cmd).await?;

    if !output.success {
        return Err(color_eyre::eyre::eyre!(
            "Failed to set env var {}: {}",
            name,
            output.stderr
        ));
    }

    println!("  Set env {} = {}", name, value);
    Ok(())
}
