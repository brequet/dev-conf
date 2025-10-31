# Flattened Codebase

Total files: 4

## Table of Contents

1. [.\common\pwsh7\profile.ps1](#file-1)
2. [.\common\scripts\bootstrap.ps1](#file-2)
3. [.\common\scripts\install-apps.ps1](#file-3)
4. [.\common\scripts\setup-configs.ps1](#file-4)

## File 1: .\common\pwsh7\profile.ps1

```ps1
$OmpTheme = "$HOME\.config\oh-my-posh\baba.omp.json"

if (Test-Path $OmpTheme) {
    oh-my-posh init pwsh --config $OmpTheme | Invoke-Expression
} else {
    Write-Warning "Oh-My-Posh theme not found at: $OmpTheme"
}

function Update-EnvironmentVariables {
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
}
Set-Alias -Name uenv -Value Update-EnvironmentVariables -ErrorAction SilentlyContinue

function cpwd {
    $pwd.Path | Set-Clipboard
}

function mkcd {
    param([string]$Path)
    New-Item -ItemType Directory -Path $Path -Force | Out-Null
    Set-Location $Path
}

function tempe {
    $tempDir = [System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), [System.IO.Path]::GetRandomFileName())
    mkcd $tempDir
}

function scratch {
    $tempFile = [System.IO.Path]::GetTempFileName()
    hx $tempFile
}

function sfx {
    param([string]$Name)
    # Possible values: bad bell dading good ringaling
    $sfxPath = "$HOME\.config\resources\sfx\$Name.ogg"
    if (Test-Path $sfxPath) {
        Start-Process -FilePath "mpv" -ArgumentList "--really-quiet", "--no-video", $sfxPath -WindowStyle Hidden
    } else {
        Write-Warning "Sound effect not found at: $sfxPath"
    }
}

function prettypath {
    $env:Path -split ';' | ForEach-Object { Write-Host $_ }
}

function catbin {
    bat "$(get-command refreshenv | Select -ExpandProperty "Source")"
}

function notify {
    param(
        [Parameter(Mandatory=$true, Position=0)]
        [string]$Message,

        [Parameter(Position=1)]
        [string]$Title = "Notification"
    )

    Add-Type -AssemblyName System.Windows.Forms
    $notification = New-Object System.Windows.Forms.NotifyIcon
    $notification.Icon = [System.Drawing.SystemIcons]::Information
    $notification.BalloonTipIcon = [System.Windows.Forms.ToolTipIcon]::Info
    $notification.BalloonTipText = $Message
    $notification.BalloonTipTitle = $Title
    $notification.Visible = $True
    $notification.ShowBalloonTip(5000)
    Start-Sleep -Seconds 10
    $notification.Dispose()
}

function timer {
    param([string]$Duration)

        if ([string]::IsNullOrWhiteSpace($Duration)) {
            Write-Warning "Usage: timer <duration> (e.g., timer 10s, timer 5m, timer 1m30s)"
            return
        }

        # Parse the duration string
        $totalSeconds = 0

        # Match patterns like "5m", "10s", "1m30s", "2h5m30s"
        if ($Duration -match '(\d+)h') {
            $totalSeconds += [int]$Matches[1] * 3600
        }
        if ($Duration -match '(\d+)m') {
            $totalSeconds += [int]$Matches[1] * 60
        }
        if ($Duration -match '(\d+)s') {
            $totalSeconds += [int]$Matches[1]
        }

        # If no unit specified, treat as seconds
        if ($Duration -match '^\d+$') {
            $totalSeconds = [int]$Duration
        }

        if ($totalSeconds -le 0) {
            Write-Warning "Invalid duration: $Duration"
            return
        }

        Write-Host "Timer started for $totalSeconds seconds ($Duration)"
        Start-Sleep -Seconds $totalSeconds
        notify "Timer completed: $Duration"
        sfx "ringaling"
}
```

## File 2: .\common\scripts\bootstrap.ps1

```ps1
#Requires -RunAsAdministrator

$ErrorActionPreference = "Stop"

$RepoRoot = $PSScriptRoot | Split-Path -Parent | Split-Path -Parent
Write-Host "Starting common setup from: $RepoRoot" -ForegroundColor Cyan

& "$PSScriptRoot\install-apps.ps1"
& "$PSScriptRoot\setup-configs.ps1"

Write-Host "`nCommon setup complete!" -ForegroundColor Green
```

## File 3: .\common\scripts\install-apps.ps1

```ps1
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

Write-Host "`nRefreshing environment PATH..." -ForegroundColor Cyan
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")

Write-Host "`nApplication installation complete!" -ForegroundColor Green
```

## File 4: .\common\scripts\setup-configs.ps1

```ps1
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
        Copy-Item -Path $mapping.Source -Destination $mapping.Target -Force -Recurse
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
```

