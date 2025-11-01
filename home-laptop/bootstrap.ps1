#Requires -RunAsAdministrator

$ErrorActionPreference = "Stop"

$RepoRoot = $PSScriptRoot | Split-Path -Parent
Write-Host "Starting home PC setup..." -ForegroundColor Cyan

Write-Host "`n=== Installing common apps and configs ===" -ForegroundColor Magenta
& "$RepoRoot\common\scripts\bootstrap.ps1"

Write-Host "`n=== Setting up home-specific configurations ===" -ForegroundColor Magenta
& "$PSScriptRoot\setup-home.ps1"

Write-Host "`nHome setup complete!" -ForegroundColor Green
