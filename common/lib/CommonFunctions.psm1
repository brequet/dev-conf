function Write-Status
{
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$Message,

        [Parameter()]
        [ValidateSet('Info', 'Success', 'Skip', 'Action', 'Warning')]
        [string]$Type = 'Info'
    )

    $colors = @{
        Info    = 'Cyan'
        Success = 'Green'
        Skip    = 'Yellow'
        Action  = 'Green'
        Warning = 'Yellow'
    }

    $prefixes = @{
        Info    = ''
        Success = ''
        Skip    = '  [SKIP] '
        Action  = '  [ACTION] '
        Warning = '  [WARNING] '
    }

    Write-Host "$($prefixes[$Type])$Message" -ForegroundColor $colors[$Type]
}

function Add-PathIfMissing
{
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$Path,

        [Parameter()]
        [ValidateSet('User', 'Machine')]
        [string]$Scope = 'User'
    )

    $currentPath = [Environment]::GetEnvironmentVariable("Path", $Scope)

    if ($currentPath -notlike "*$Path*")
    {
        Write-Status "Adding $Path to $Scope PATH" -Type Action
        [Environment]::SetEnvironmentVariable("Path", "$currentPath;$Path", $Scope)
        return $true
    }

    return $false
}

function Update-EnvironmentPath
{
    [CmdletBinding()]
    param()

    Write-Status "Refreshing environment PATH..." -Type Info
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" +
    [System.Environment]::GetEnvironmentVariable("Path", "User")
}

function New-DirectoryIfMissing
{
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$Path
    )

    if (-not (Test-Path $Path))
    {
        Write-Status "Creating directory: $Path" -Type Action
        New-Item -ItemType Directory -Path $Path -Force | Out-Null
        return $true
    }

    return $false
}

function Install-ConfigFile
{
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$Source,

        [Parameter(Mandatory)]
        [string]$Target,

        [Parameter(Mandatory)]
        [ValidateSet('Copy', 'SymbolicLink')]
        [string]$Type,

        [Parameter()]
        [switch]$CreateTargetDir
    )

    # Validate source exists
    if (-not (Test-Path $Source))
    {
        Write-Status "Source not found: $Source" -Type Warning
        return $false
    }

    # Create target directory if needed
    if ($CreateTargetDir)
    {
        $targetDir = Split-Path -Parent $Target
        if ($targetDir)
        {
            $null = New-DirectoryIfMissing -Path $targetDir
        }
    }

    # Remove existing target
    if (Test-Path $Target)
    {
        Write-Status "Removing existing: $Target" -Type Action
        Remove-Item $Target -Force -Recurse -ErrorAction SilentlyContinue
    }

    # Install based on type
    if ($Type -eq "SymbolicLink")
    {
        Write-Status "[SYMLINK] $Target -> $Source" -Type Action
        New-Item -ItemType SymbolicLink -Path $Target -Target $Source -Force | Out-Null
    } else
    {
        Write-Status "[COPY] $Source -> $Target" -Type Action
        Copy-Item -Path $Source -Destination $Target -Force -Recurse
    }

    return $true
}

function Set-UserEnvironmentVariable
{
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$Name,

        [Parameter(Mandatory)]
        [string]$Value,

        [Parameter()]
        [ValidateSet('User', 'Machine')]
        [string]$Scope = 'User'
    )

    Write-Status "[ENV] Setting $Name = $Value" -Type Action
    [Environment]::SetEnvironmentVariable($Name, $Value, $Scope)
}

Export-ModuleMember -Function @(
    'Write-Status',
    'Add-PathIfMissing',
    'Update-EnvironmentPath',
    'New-DirectoryIfMissing',
    'Install-ConfigFile',
    'Set-UserEnvironmentVariable'
)
