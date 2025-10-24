#Requires -RunAsAdministrator

$ErrorActionPreference = "Stop"

$RepoRoot = $PSScriptRoot | Split-Path -Parent | Split-Path -Parent
Write-Host "Starting common setup from: $RepoRoot" -ForegroundColor Cyan

& "$PSScriptRoot\install-apps.ps1"
& "$PSScriptRoot\setup-configs.ps1"

Write-Host "`nCommon setup complete!" -ForegroundColor Green
