use color_eyre::Result;
use tokio::process::Command;

/// Output from a shell command
pub struct ShellOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Run a PowerShell command and capture output
pub async fn run_powershell(script: &str) -> Result<ShellOutput> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ])
        .output()
        .await?;

    Ok(ShellOutput {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}
