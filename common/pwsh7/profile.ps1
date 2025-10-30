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
