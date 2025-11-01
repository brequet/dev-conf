$ErrorActionPreference = "Stop"

$RepoRoot = $PSScriptRoot | Split-Path -Parent

Write-Host "Configuring home PC specifics..." -ForegroundColor Cyan

$pwsh7ProfileDir = "$HOME\Documents\PowerShell"

$homeAliasesSource = "$RepoRoot\home-pc\pwsh7\aliases.ps1"
$homeAliasesTarget = "$pwsh7ProfileDir\home-aliases.ps1"

if (Test-Path $homeAliasesSource) {
    Write-Host "  [SYMLINK] Home aliases -> $homeAliasesTarget" -ForegroundColor Green

    if (Test-Path $homeAliasesTarget) {
        Remove-Item $homeAliasesTarget -Force
    }

    New-Item -ItemType SymbolicLink -Path $homeAliasesTarget -Target $homeAliasesSource -Force | Out-Null

    $profilePath = "$pwsh7ProfileDir\Microsoft.PowerShell_profile.ps1"
    $dotCommand = ". `"$homeAliasesTarget`""

    if (Test-Path $profilePath) {
        $profileContent = Get-Content $profilePath -Raw
        if ($profileContent -notmatch [regex]::Escape($dotCommand)) {
            Write-Host "  Adding home aliases to profile..." -ForegroundColor Yellow
            Add-Content -Path $profilePath -Value "`n$dotCommand`n"
        }
    }
}

Write-Host "`nHome PC configuration complete!" -ForegroundColor Green
