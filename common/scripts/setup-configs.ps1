#Requires -RunAsAdministrator

<#
.SYNOPSIS
    Sets up configuration files and environment variables.

.DESCRIPTION
    This script installs configuration files (via symlinks or copies),
    sets up environment variables, and configures system language settings.
#>

$ErrorActionPreference = "Stop"

Import-Module "$PSScriptRoot\..\lib\CommonFunctions.psm1" -Force

#region Configuration Data

$script:RepoRoot = $PSScriptRoot | Split-Path -Parent | Split-Path -Parent

# Configuration file mappings
$script:ConfigMappings = @(
    @{
        Source = "$script:RepoRoot\common\pwsh7\profile.ps1"
        Target = "$HOME\Documents\PowerShell\Microsoft.PowerShell_profile.ps1"
        Type = "SymbolicLink"
        CreateDir = $true
    }
    @{
        Source = "$script:RepoRoot\common\zed\settings.json"
        Target = "$env:APPDATA\Zed\settings.json"
        Type = "Copy"
        CreateDir = $true
    }
    @{
        Source = "$script:RepoRoot\common\oh-my-posh\baba.omp.json"
        Target = "$HOME\.config\oh-my-posh\baba.omp.json"
        Type = "SymbolicLink"
        CreateDir = $true
    }
    @{
        Source = "$script:RepoRoot\common\resources"
        Target = "$HOME\.config\resources"
        Type = "Copy"
        CreateDir = $true
    }
)

# Environment variables to set
$script:EnvironmentVariables = @(
    @{Name = "ZED_ALLOW_EMULATED_GPU"; Value = "1"}
)

# System language to configure
$script:SystemLanguage = "fr-FR"

#endregion

#region Helper Functions

function Install-ConfigurationFiles
{
    <#
    .SYNOPSIS
        Installs all configuration files based on the defined mappings.
    #>
    [CmdletBinding()]
    param()

    Write-Status "Setting up configuration files..." -Type Info

    foreach ($mapping in $script:ConfigMappings)
    {
        $params = @{
            Source = $mapping.Source
            Target = $mapping.Target
            Type = $mapping.Type
        }

        if ($mapping.CreateDir)
        {
            $params.CreateTargetDir = $true
        }

        Install-ConfigFile @params
    }
}

function Set-EnvironmentVariables
{
    <#
    .SYNOPSIS
        Sets all required environment variables.
    #>
    [CmdletBinding()]
    param()

    if ($script:EnvironmentVariables.Count -eq 0)
    {
        Write-Status "No environment variables to set" -Type Skip
        return
    }

    Write-Status "Setting up environment variables..." -Type Info

    foreach ($envVar in $script:EnvironmentVariables)
    {
        Set-UserEnvironmentVariable -Name $envVar.Name -Value $envVar.Value
    }
}

function Set-SystemLanguage
{
    <#
    .SYNOPSIS
        Configures the Windows system language settings.
    #>
    [CmdletBinding()]
    param()

    if (-not $script:SystemLanguage)
    {
        Write-Status "System language configuration skipped" -Type Skip
        return
    }

    Write-Status "Setting up system language: $script:SystemLanguage..." -Type Info

    try
    {
        Set-WinUserLanguageList -LanguageList $script:SystemLanguage -Force
        Write-Status "System language configured successfully" -Type Success
    } catch
    {
        Write-Status "Failed to set language: $_" -Type Warning
    }
}

#endregion

#region Main Execution

function Invoke-ConfigSetup
{
    <#
    .SYNOPSIS
        Main entry point for configuration setup.
    #>
    [CmdletBinding()]
    param()

    Write-Status "Starting configuration setup from: $script:RepoRoot" -Type Info
    Write-Host ""

    Install-ConfigurationFiles
    Write-Host ""

    Set-EnvironmentVariables
    Write-Host ""

    Set-SystemLanguage
    Write-Host ""

    Write-Status "Configuration setup complete!" -Type Success
}

# Execute main function
Invoke-ConfigSetup

#endregion
