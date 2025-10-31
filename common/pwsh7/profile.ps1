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
