#Requires -RunAsAdministrator

<#
.SYNOPSIS
    Installs development applications from various sources.

.DESCRIPTION
    This script installs applications from winget and GitHub releases.
#>

$ErrorActionPreference = "Stop"

Import-Module "$PSScriptRoot\..\lib\CommonFunctions.psm1" -Force

#region Configuration Data

# Applications to install from winget
$script:WingetApps = @(
    @{Id = "Microsoft.WindowsTerminal"; Name = "Windows Terminal"}
    @{Id = "Microsoft.PowerShell"; Name = "PowerShell 7"}
    @{Id = "sharkdp.bat"; Name = "bat"}
    @{Id = "sharkdp.fd"; Name = "fd"}
    @{Id = "junegunn.fzf"; Name = "fzf"}
    @{Id = "Microsoft.VisualStudioCode"; Name = "VSCode"}
    @{Id = "ZedIndustries.Zed"; Name = "Zed"}
    @{Id = "BurntSushi.ripgrep.MSVC"; Name = "ripgrep"}
    @{Id = "Helix.Helix"; Name = "Helix"}
    @{Id = "DEVCOM.JetBrainsMonoNerdFont"; Name = "JetBrains Mono Nerd Font"}
    @{Id = "eza-community.eza"; Name = "eza"}
    @{Id = "WinDirStat.WinDirStat"; Name = "WinDirStat"}
    @{Id = "mpv.net"; Name = "mpv"}
    @{Id = "Starship.Starship"; Name = "Starship Prompt"}
    @{Id = "GitHub.Copilot"; Name = "GitHub Copilot CLI"}
)

# Applications to install from GitHub releases
$script:GitHubApps = @(
    @{Repository = "brequet/flatten"; AssetPattern = "flatten-windows-x86_64.exe"; Target = "flatten.exe"}
)

# Custom binary directory
$script:BinPath = "$HOME\.config\bin"

# Additional paths to add to PATH environment variable
$script:AdditionalPaths = @(
    "$env:LOCALAPPDATA\Programs\mpv.net"
)

#endregion

#region Helper Functions

function Test-WingetAppInstalled
{
    <#
    .SYNOPSIS
        Checks if a winget application is already installed.
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$AppId
    )

    $null = winget list --id $AppId --exact 2>&1
    return $LASTEXITCODE -eq 0
}

function Install-WingetApp
{
    <#
    .SYNOPSIS
        Installs a single application using winget.
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$Id,

        [Parameter(Mandatory)]
        [string]$Name
    )

    if (Test-WingetAppInstalled -AppId $Id)
    {
        Write-Status "$Name already installed" -Type Skip
    } else
    {
        Write-Status "$Name..." -Type Action
        winget install --id=$Id -e --silent --accept-source-agreements --accept-package-agreements
    }
}

function Get-GitHubReleaseAsset
{
    <#
    .SYNOPSIS
        Gets the latest release asset from a GitHub repository.
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$Repository,

        [Parameter(Mandatory)]
        [string]$AssetPattern
    )

    $apiUrl = "https://api.github.com/repos/$Repository/releases/latest"
    $release = Invoke-RestMethod -Uri $apiUrl -Method Get
    $asset = $release.assets | Where-Object { $_.name -like $AssetPattern } | Select-Object -First 1

    return $asset
}

function Install-GitHubApp
{
    <#
    .SYNOPSIS
        Installs an application from a GitHub release.
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$Repository,

        [Parameter(Mandatory)]
        [string]$AssetPattern,

        [Parameter(Mandatory)]
        [string]$Target,

        [Parameter(Mandatory)]
        [string]$InstallPath
    )

    $targetPath = Join-Path -Path $InstallPath -ChildPath $Target

    if (Test-Path $targetPath)
    {
        Write-Status "$Target already exists" -Type Skip
        return
    }

    Write-Status "Downloading $Target..." -Type Action

    $asset = Get-GitHubReleaseAsset -Repository $Repository -AssetPattern $AssetPattern

    if (-not $asset)
    {
        Write-Status "Asset matching '$AssetPattern' not found in $Repository" -Type Warning
        return
    }

    $tempFile = Join-Path -Path $env:TEMP -ChildPath $asset.name
    (New-Object Net.WebClient).DownloadFile($asset.browser_download_url, $tempFile)
    Move-Item -Path $tempFile -Destination $targetPath -Force
}

#endregion

#region Installation Functions

function Initialize-Environment
{
    <#
    .SYNOPSIS
        Initializes the installation environment.
    #>
    [CmdletBinding()]
    param()

    Write-Status "Accepting winget source agreements..." -Type Info
    $null = winget list --accept-source-agreements 2>&1
}

function Install-WingetApps
{
    <#
    .SYNOPSIS
        Installs all applications from the winget apps list.
    #>
    [CmdletBinding()]
    param()

    Write-Status "Installing applications from winget..." -Type Info

    foreach ($app in $script:WingetApps)
    {
        Install-WingetApp -Id $app.Id -Name $app.Name
    }
}

function Install-GitHubApps
{
    <#
    .SYNOPSIS
        Installs all applications from the GitHub apps list.
    #>
    [CmdletBinding()]
    param()

    # Create and add bin directory to PATH
    if (-not (Test-Path $script:BinPath))
    {
        Write-Status "Creating $script:BinPath..." -Type Action
        New-Item -Path $script:BinPath -ItemType Directory -Force | Out-Null
    }

    $null = Add-PathIfMissing -Path $script:BinPath

    Write-Status "Installing applications from GitHub..." -Type Info

    foreach ($app in $script:GitHubApps)
    {
        Install-GitHubApp -Repository $app.Repository `
            -AssetPattern $app.AssetPattern `
            -Target $app.Target `
            -InstallPath $script:BinPath
    }
}

function Add-AdditionalPaths
{
    <#
    .SYNOPSIS
        Adds additional paths to the PATH environment variable.
    #>
    [CmdletBinding()]
    param()

    foreach ($path in $script:AdditionalPaths)
    {
        if (Test-Path $path)
        {
            $null = Add-PathIfMissing -Path $path
        }
    }
}

#endregion

#region Main Execution

function Invoke-AppInstallation
{
    <#
    .SYNOPSIS
        Main entry point for application installation.
    #>
    [CmdletBinding()]
    param()

    Initialize-Environment
    Install-WingetApps
    Add-AdditionalPaths
    Install-GitHubApps
    Update-EnvironmentPath

    Write-Status "Application installation complete!" -Type Success
}

# Execute main function
Invoke-AppInstallation

#endregion
