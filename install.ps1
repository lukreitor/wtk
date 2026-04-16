#Requires -Version 5.1
<#
.SYNOPSIS
    WTK (Windows Token Killer) - One-click installer
.DESCRIPTION
    Downloads and installs WTK with automatic PATH configuration.
    Run: irm https://raw.githubusercontent.com/Lukreitor/wtk/master/install.ps1 | iex
.NOTES
    Author: WTK Contributors
    License: MIT
#>

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

# Configuration
$repo = "Lukreitor/wtk"
$installDir = "$env:LOCALAPPDATA\wtk"
$exeName = "wtk.exe"

Write-Host ""
Write-Host "  WTK - Windows Token Killer Installer" -ForegroundColor Cyan
Write-Host "  =====================================" -ForegroundColor Cyan
Write-Host ""

# Get latest release
Write-Host "  [1/4] Fetching latest release..." -ForegroundColor Yellow
try {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest"
    $version = $release.tag_name
    $asset = $release.assets | Where-Object { $_.name -like "*windows*x64*.zip" } | Select-Object -First 1

    if (-not $asset) {
        throw "No Windows x64 asset found in release"
    }

    $downloadUrl = $asset.browser_download_url
    Write-Host "       Found version: $version" -ForegroundColor Green
} catch {
    Write-Host "       Failed to fetch release: $_" -ForegroundColor Red
    exit 1
}

# Download
Write-Host "  [2/4] Downloading WTK..." -ForegroundColor Yellow
$tempZip = "$env:TEMP\wtk-$version.zip"
try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tempZip -UseBasicParsing
    Write-Host "       Downloaded successfully" -ForegroundColor Green
} catch {
    Write-Host "       Download failed: $_" -ForegroundColor Red
    exit 1
}

# Install
Write-Host "  [3/4] Installing to $installDir..." -ForegroundColor Yellow
try {
    # Create install directory
    if (Test-Path $installDir) {
        Remove-Item -Path $installDir -Recurse -Force
    }
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null

    # Extract
    Expand-Archive -Path $tempZip -DestinationPath $installDir -Force

    # Handle nested directory (if zip contains a folder)
    $nestedExe = Get-ChildItem -Path $installDir -Recurse -Filter $exeName | Select-Object -First 1
    if ($nestedExe -and $nestedExe.DirectoryName -ne $installDir) {
        Move-Item -Path "$($nestedExe.DirectoryName)\*" -Destination $installDir -Force
    }

    # Verify exe exists
    if (-not (Test-Path "$installDir\$exeName")) {
        throw "wtk.exe not found after extraction"
    }

    # Cleanup
    Remove-Item -Path $tempZip -Force -ErrorAction SilentlyContinue

    Write-Host "       Installed successfully" -ForegroundColor Green
} catch {
    Write-Host "       Installation failed: $_" -ForegroundColor Red
    exit 1
}

# Add to PATH
Write-Host "  [4/4] Configuring PATH..." -ForegroundColor Yellow
try {
    $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($userPath -notlike "*$installDir*") {
        [Environment]::SetEnvironmentVariable("PATH", "$userPath;$installDir", "User")
        $env:PATH = "$env:PATH;$installDir"
        Write-Host "       Added to PATH" -ForegroundColor Green
    } else {
        Write-Host "       Already in PATH" -ForegroundColor Green
    }
} catch {
    Write-Host "       Failed to update PATH: $_" -ForegroundColor Red
    Write-Host "       Please add manually: $installDir" -ForegroundColor Yellow
}

# Verify installation
Write-Host ""
Write-Host "  Installation complete!" -ForegroundColor Green
Write-Host ""

try {
    $wtkVersion = & "$installDir\$exeName" --version 2>&1
    Write-Host "  Installed: $wtkVersion" -ForegroundColor Cyan
} catch {
    Write-Host "  Installed: wtk $version" -ForegroundColor Cyan
}

Write-Host ""
Write-Host "  Quick Start:" -ForegroundColor Yellow
Write-Host "    wtk git status      # Compressed git output"
Write-Host "    wtk gain            # View token savings"
Write-Host "    wtk gain --graph    # ASCII savings graph"
Write-Host "    wtk init --claude-code  # Setup Claude Code hooks"
Write-Host ""
Write-Host "  NOTE: Restart your terminal for PATH changes to take effect" -ForegroundColor Magenta
Write-Host ""
