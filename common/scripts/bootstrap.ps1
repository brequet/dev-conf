#Requires -RunAsAdministrator

$ErrorActionPreference = "Stop"

Import-Module "$PSScriptRoot\..\lib\CommonFunctions.psm1" -Force

function Initialize-Setup
{
    [CmdletBinding()]
    param()

    $repoRoot = $PSScriptRoot | Split-Path -Parent | Split-Path -Parent

    Write-Status "==================================================" -Type Info
    Write-Status "  Common Development Environment Setup" -Type Info
    Write-Status "==================================================" -Type Info
    Write-Host ""
    Write-Status "Repository Root: $repoRoot" -Type Info
    Write-Status "PowerShell Version: $($PSVersionTable.PSVersion)" -Type Info
    Write-Host ""

    return $repoRoot
}

function Invoke-SetupScript
{
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$ScriptPath,

        [Parameter(Mandatory)]
        [string]$Description
    )

    Write-Status "--------------------------------------------------" -Type Info
    Write-Status "  $Description" -Type Info
    Write-Status "--------------------------------------------------" -Type Info
    Write-Host ""

    try
    {
        & $ScriptPath

        if ($LASTEXITCODE -and $LASTEXITCODE -ne 0)
        {
            Write-Status "Script exited with code: $LASTEXITCODE" -Type Warning
            return $false
        }

        Write-Host ""
        Write-Status "$Description completed successfully" -Type Success
        Write-Host ""
        return $true
    } catch
    {
        Write-Status "Error executing script: $_" -Type Warning
        Write-Host ""
        return $false
    }
}

function Complete-Setup
{
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [bool]$Success
    )

    Write-Host ""

    if ($Success)
    {
        Write-Status "==================================================" -Type Success
        Write-Status "  Common Setup Complete!" -Type Success
        Write-Status "==================================================" -Type Success
    } else
    {
        Write-Status "==================================================" -Type Warning
        Write-Status "  Setup Completed with Warnings" -Type Warning
        Write-Status "==================================================" -Type Warning
        Write-Host ""
        Write-Status "Some steps may have failed. Please review the output above." -Type Warning
    }

    Write-Host ""
}

function Invoke-Bootstrap
{
    [CmdletBinding()]
    param()

    $null = Initialize-Setup
    $allSuccess = $true

    # Step 1: Install Applications
    $scriptPath = "$PSScriptRoot\install-apps.ps1"
    if (Test-Path $scriptPath)
    {
        $result = Invoke-SetupScript -ScriptPath $scriptPath -Description "Installing Applications"
        $allSuccess = $allSuccess -and $result
    } else
    {
        Write-Status "Script not found: $scriptPath" -Type Warning
        $allSuccess = $false
    }

    # Step 2: Setup Configurations
    $scriptPath = "$PSScriptRoot\setup-configs.ps1"
    if (Test-Path $scriptPath)
    {
        $result = Invoke-SetupScript -ScriptPath $scriptPath -Description "Setting Up Configurations"
        $allSuccess = $allSuccess -and $result
    } else
    {
        Write-Status "Script not found: $scriptPath" -Type Warning
        $allSuccess = $false
    }

    # Complete setup
    Complete-Setup -Success $allSuccess
}

Invoke-Bootstrap
