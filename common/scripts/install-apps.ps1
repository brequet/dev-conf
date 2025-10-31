#Requires -RunAsAdministrator

$ErrorActionPreference = "Stop"

Write-Host "Accepting winget source agreements..." -ForegroundColor Cyan
winget list --accept-source-agreements | Out-Null

function Test-AppInstalled {
    param([string]$AppId)
    $result = winget list --id $AppId --exact 2>$null
    return $LASTEXITCODE -eq 0
}

$wingetApps = @(
    @{Id = "Microsoft.WindowsTerminal"; Name = "Windows Terminal"},
    @{Id = "Microsoft.PowerShell"; Name = "PowerShell 7"},
    @{Id = "JanDeDobbeleer.OhMyPosh"; Name = "OhMyPosh"},
    @{Id = "sharkdp.bat"; Name = "bat"},
    @{Id = "sharkdp.fd"; Name = "fd"},
    @{Id = "junegunn.fzf"; Name = "fzf"},
    @{Id = "Microsoft.VisualStudioCode"; Name = "VSCode"},
    @{Id = "ZedIndustries.Zed"; Name = "Zed"},
    @{Id = "BurntSushi.ripgrep.MSVC"; Name = "ripgrep"},
    @{Id = "Helix.Helix"; Name = "Helix"},
    @{Id = "DEVCOM.JetBrainsMonoNerdFont"; Name = "JetBrains Mono Nerd Font"},
    @{Id = "eza-community.eza"; Name = "eza"},
    @{Id = "WinDirStat.WinDirStat"; Name = "WinDirStat"},
    @{Id = "mpv.net"; Name = "mpv"}
)

Write-Host "Installing common applications..." -ForegroundColor Cyan

foreach ($app in $wingetApps) {
    if (Test-AppInstalled -AppId $app.Id) {
        Write-Host "  [SKIP] $($app.Name) already installed" -ForegroundColor Yellow
    } else {
        Write-Host "  [INSTALL] $($app.Name)..." -ForegroundColor Green
        winget install --id=$($app.Id) -e --silent --accept-source-agreements --accept-package-agreements
    }
}

# Special case for mpv as it does not put itself available in the path
$mpvPath = "$env:LOCALAPPDATA\Programs\mpv.net"
if (Test-Path $mpvPath) {
    $currentUserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentUserPath -notlike "*$mpvPath*") {
        Write-Host "  [PATH] Adding mpv.net to User PATH" -ForegroundColor Green
        [Environment]::SetEnvironmentVariable("Path", "$currentUserPath;$mpvPath", "User")
    }
}

Write-Host "`nRefreshing environment PATH..." -ForegroundColor Cyan
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")

Write-Host "`nApplication installation complete!" -ForegroundColor Green
