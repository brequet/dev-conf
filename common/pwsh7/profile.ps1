$OmpTheme = "$HOME\.config\oh-my-posh\baba.omp.json"

if (Test-Path $OmpTheme) {
    oh-my-posh init pwsh --config $OmpTheme | Invoke-Expression
} else {
    Write-Warning "Oh-My-Posh theme not found at: $OmpTheme"
}

function Update-EnvironmentVariables {
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
}

function mkcd {
    param([string]$Path)
    New-Item -ItemType Directory -Path $Path -Force | Out-Null
    Set-Location $Path
}
