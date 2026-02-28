# install.ps1 - Bootstrap devconf on a fresh Windows machine
# Usage: irm https://raw.githubusercontent.com/brequet/dev-conf/main/install.ps1 | iex
#   or:  .\install.ps1

$ErrorActionPreference = "Stop"

$repo = "brequet/dev-conf"
$binName = "devconf.exe"
$installDir = "$env:USERPROFILE\.config\bin"

Write-Host "Installing devconf..." -ForegroundColor Cyan

# Create install directory
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

# Get latest release info from GitHub API
$releaseUrl = "https://api.github.com/repos/$repo/releases/latest"
try {
    $release = Invoke-RestMethod -Uri $releaseUrl -Headers @{ "User-Agent" = "devconf-installer" }
} catch {
    Write-Host "Failed to fetch latest release from GitHub." -ForegroundColor Red
    Write-Host "Error: $_" -ForegroundColor Red
    exit 1
}

$tag = $release.tag_name
Write-Host "Latest version: $tag" -ForegroundColor Green

# Find the devconf.exe asset
$asset = $release.assets | Where-Object { $_.name -eq $binName }
if (-not $asset) {
    Write-Host "Could not find $binName in release $tag" -ForegroundColor Red
    exit 1
}

$downloadUrl = $asset.browser_download_url
$destPath = Join-Path $installDir $binName

Write-Host "Downloading $downloadUrl..."
Invoke-WebRequest -Uri $downloadUrl -OutFile $destPath -UseBasicParsing

# Add to PATH if not already there
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$installDir*") {
    Write-Host "Adding $installDir to user PATH..."
    [Environment]::SetEnvironmentVariable("PATH", "$userPath;$installDir", "User")
    $env:PATH = "$env:PATH;$installDir"
}

Write-Host ""
Write-Host "devconf $tag installed to $destPath" -ForegroundColor Green
Write-Host "Run 'devconf --help' to get started." -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Clone your dev-conf repo"
Write-Host "  2. cd into the repo directory"
Write-Host "  3. Run 'devconf profile set <profile-name>'"
Write-Host "  4. Run 'devconf' to launch the TUI"
