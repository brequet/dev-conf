# VM Setup

## Env Variables

`WIN_USER`, `WIN_PASSWORD` and `GITLAB_TOKEN`

Set `ZED_ALLOW_EMULATED_GPU=1`

## Create a persistent network drive to S: for the shared folder

cf scripts. Maybe can do better ?

```powershell
# Script to map a persistent network drive with credentials from environment variables.

# --- Configuration ---
$driveLetter = "S"
$networkPath = "\FRL205143\shared"

# --- Credentials from Environment Variables ---
$username = $env:WIN_USER
$password = $env:WIN_PASSWORD

# --- Script Logic ---
if (-not ($username -and $password)) {
    Write-Error "Error: WIN_USER and WIN_PASSWORD environment variables must be set."
    return
}

if (Test-Path -Path "${driveLetter}:") {
    Write-Host "Drive ${driveLetter}: is already mapped. NTD."
} else {
    try {
        Write-Host "Attempting to map ${driveLetter}: to ${networkPath}..."

        $securePassword = ConvertTo-SecureString -String $password -AsPlainText -Force
        $credential = New-Object System.Management.Automation.PSCredential($username, $securePassword)

        New-PSDrive -Name $driveLetter -PSProvider FileSystem -Root $networkPath -Credential $credential -Persist -ErrorAction Stop

        Write-Host "Successfully mapped ${driveLetter}: to ${networkPath}. The mapping is persistent."
    }
    catch {
        Write-Error "Failed to map network drive. Error: $_"
    }
}
```
