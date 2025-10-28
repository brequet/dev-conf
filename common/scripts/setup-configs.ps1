$ErrorActionPreference = "Stop"

$RepoRoot = $PSScriptRoot | Split-Path -Parent | Split-Path -Parent

Write-Host "Setting up configurations..." -ForegroundColor Cyan

$pwsh7ProfileDir = "$HOME\Documents\PowerShell"
$zedConfigPath = "$env:APPDATA\Zed"

$configMappings = @(
    @{
        Source = "$RepoRoot\common\pwsh7\profile.ps1"
        Target = "$pwsh7ProfileDir\Microsoft.PowerShell_profile.ps1"
        Type = "SymbolicLink"
        CreateDir = $true
    },
    @{
        Source = "$RepoRoot\common\zed\settings.json"
        Target = "$zedConfigPath\settings.json"
        Type = "Copy"
        CreateDir = $true
    },
    @{
        Source = "$RepoRoot\common\oh-my-posh\baba.omp.json"
        Target = "$HOME\.config\oh-my-posh\baba.omp.json"
        Type = "SymbolicLink"
        CreateDir = $true
    },
    @{
        Source = "$RepoRoot\common\resources"
        Target = "$HOME\.config\resources"
        Type = "Copy"
        CreateDir = $true
    }
)

foreach ($mapping in $configMappings) {
    if ($mapping.CreateDir) {
        $targetDir = Split-Path -Parent $mapping.Target
        if (!(Test-Path $targetDir)) {
            Write-Host "  Creating directory: $targetDir" -ForegroundColor Yellow
            New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
        }
    }

    if (Test-Path $mapping.Target) {
        Write-Host "  Removing existing: $($mapping.Target)" -ForegroundColor Yellow
        Remove-Item $mapping.Target -Force -Recurse
    }

    if ($mapping.Type -eq "SymbolicLink") {
        Write-Host "  [SYMLINK] $($mapping.Target) -> $($mapping.Source)" -ForegroundColor Green
        New-Item -ItemType SymbolicLink -Path $mapping.Target -Target $mapping.Source -Force | Out-Null
    } else {
        Write-Host "  [COPY] $($mapping.Source) -> $($mapping.Target)" -ForegroundColor Green
        Copy-Item -Path $mapping.Source -Destination $mapping.Target -Force
    }
}

Write-Host "Setting up environment variables..." -ForegroundColor Cyan

$envVars = @(
    @{
        Key = "ZED_ALLOW_EMULATED_GPU"
        Value = 1
    }
)

foreach ($envVar in $envVars) {
    Write-Host "  [ENV] Setting $($envVar.Key) = $($envVar.Value)" -ForegroundColor Green
    [Environment]::SetEnvironmentVariable($envVar.Key, $envVar.Value, "User")
}

Write-Host "Setting up French language..." -ForegroundColor Cyan

Set-WinUserLanguageList -LanguageList fr-FR -Force

Write-Host "`nConfiguration setup complete!" -ForegroundColor Green
