#!/usr/bin/env pwsh
# Wenget Remote Installation Script for Windows
# Usage: irm https://raw.githubusercontent.com/superyngo/Wenget/main/install.ps1 | iex

param(
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

# Colors
function Write-Success { Write-Host $args -ForegroundColor Green }
function Write-Info { Write-Host $args -ForegroundColor Cyan }
function Write-Error { Write-Host $args -ForegroundColor Red }
function Write-Warning { Write-Host $args -ForegroundColor Yellow }

# Configuration
$APP_NAME = "wenget"
$REPO = "superyngo/Wenget"

# Detect if running as Administrator for system-level installation
function Get-InstallMode {
    $isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

    if ($isAdmin) {
        $script:SYSTEM_INSTALL = $true
        $programFiles = $env:ProgramW6432
        if (-not $programFiles) {
            $programFiles = $env:ProgramFiles
        }
        $script:WENGET_HOME = "$programFiles\wenget"
        $script:INSTALL_DIR = "$programFiles\wenget\app\wenget"
        $script:BIN_DIR = "$programFiles\wenget\bin"
        $script:BIN_PATH = "$script:INSTALL_DIR\$APP_NAME.exe"
        Write-Info "Running as Administrator - using system-level installation"
        Write-Info "  App directory: $programFiles\wenget\app"
        Write-Info "  Bin directory: $programFiles\wenget\bin"
    } else {
        $script:SYSTEM_INSTALL = $false
        $script:WENGET_HOME = "$env:USERPROFILE\.wenget"
        $script:INSTALL_DIR = "$env:USERPROFILE\.wenget\apps\wenget"
        $script:BIN_DIR = "$env:USERPROFILE\.wenget\bin"
        $script:BIN_PATH = "$script:INSTALL_DIR\$APP_NAME.exe"
        Write-Info "Running as user - using user-level installation"
        Write-Info "  App directory: $env:USERPROFILE\.wenget\apps"
        Write-Info "  Bin directory: $env:USERPROFILE\.wenget\bin"
    }
    Write-Info ""
}

function Get-LatestRelease {
    try {
        $apiUrl = "https://api.github.com/repos/$REPO/releases/latest"
        Write-Info "Fetching latest release information..."

        $release = Invoke-RestMethod -Uri $apiUrl -Headers @{
            "User-Agent" = "wenget-installer"
        }

        return $release
    } catch {
        Write-Error "Failed to fetch release information: $_"
        exit 1
    }
}

function Get-Architecture {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { return "x86_64" }
        "x86" { return "i686" }
        "ARM64" { return "aarch64" }
        default {
            Write-Warning "Unknown architecture: $arch, defaulting to x86_64"
            return "x86_64"
        }
    }
}

function Install-Wenget {
    Write-Info "=== Wenget Installation Script ==="
    Write-Info ""

    # Detect install mode first
    Get-InstallMode

    # Get latest release
    $release = Get-LatestRelease
    $version = $release.tag_name
    Write-Success "Latest version: $version"

    # Determine architecture
    $arch = Get-Architecture
    Write-Info "Detected architecture: $arch"

    # Find download URL for Windows
    $assetName = "$APP_NAME-windows-$arch.exe"
    $asset = $release.assets | Where-Object { $_.name -eq $assetName }

    if (-not $asset) {
        Write-Error "Could not find Windows release asset"
        Write-Info "Available assets:"
        $release.assets | ForEach-Object { Write-Info "  - $($_.name)" }
        Write-Info ""
        Write-Info "Looking for: $assetName"
        exit 1
    }

    $downloadUrl = $asset.browser_download_url
    Write-Info "Download URL: $downloadUrl"
    Write-Info ""

    # Create installation directory
    if (-not (Test-Path $INSTALL_DIR)) {
        New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null
    }

    # Download binary directly
    Write-Info "Downloading $APP_NAME..."

    $ProgressPreference = 'SilentlyContinue'
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $BIN_PATH -UseBasicParsing
        $ProgressPreference = 'Continue'
        Write-Success "Downloaded successfully!"
    } catch {
        $ProgressPreference = 'Continue'
        Write-Error "Download failed: $_"
        exit 1
    }

    Write-Info "Installed to: $INSTALL_DIR"
    Write-Success "Binary installed successfully!"
    Write-Info ""

    # For system-level installation, create bin directory and add to PATH
    if ($SYSTEM_INSTALL) {
        # Create bin directory
        if (-not (Test-Path $BIN_DIR)) {
            New-Item -ItemType Directory -Path $BIN_DIR -Force | Out-Null
        }

        # Create a copy or shim in bin directory
        $shimPath = "$BIN_DIR\$APP_NAME.exe"
        Copy-Item -Path $BIN_PATH -Destination $shimPath -Force
        Write-Info "Copied to bin directory: $shimPath"

        # Add bin directory to system PATH if not already present
        $currentPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::Machine)
        if ($currentPath -notlike "*$BIN_DIR*") {
            Write-Info "Adding $BIN_DIR to system PATH..."
            $newPath = "$currentPath;$BIN_DIR"
            [Environment]::SetEnvironmentVariable("Path", $newPath, [EnvironmentVariableTarget]::Machine)
            Write-Success "Added to system PATH"
        } else {
            Write-Info "Bin directory already in system PATH"
        }
        Write-Info ""
    }

    # Run wenget init
    Write-Info "Initializing Wenget..."
    Write-Info ""

    try {
        & $BIN_PATH init --yes
        Write-Info ""
        Write-Success "Wenget initialized successfully!"
    } catch {
        Write-Warning "Failed to run wenget init. You can run it manually later."
        Write-Info "  Run: wenget init"
    }

    Write-Info ""
    Write-Success "Installation completed successfully!"
    Write-Info ""
    Write-Info "Installed version: $version"
    Write-Info "Installation path: $BIN_PATH"
    Write-Info ""
    Write-Info "Usage:"
    Write-Info "  wenget search <keyword>     - Search packages"
    Write-Info "  wenget add <package>        - Install a package"
    Write-Info "  wenget list                 - List installed packages"
    Write-Info "  wenget --help               - Show help"
    Write-Info ""
    Write-Warning "Note: You may need to restart your terminal for PATH changes to take effect."
    Write-Info ""
    Write-Info "To uninstall, run:"
    Write-Info "  irm https://raw.githubusercontent.com/$REPO/main/install.ps1 | iex -Uninstall"
}

function Uninstall-Wenget {
    Write-Info "=== Wenget Uninstallation Script ==="
    Write-Info ""

    # Detect install mode first
    Get-InstallMode

    # Check if wenget is available and run self-deletion
    if (Test-Path $BIN_PATH) {
        Write-Info "Running Wenget self-deletion..."
        try {
            & $BIN_PATH del self --yes
            Write-Success "Wenget uninstalled successfully!"
        } catch {
            Write-Warning "Wenget self-deletion failed. Performing manual cleanup..."

            # For system-level installation, remove from PATH and bin directory
            if ($SYSTEM_INSTALL) {
                # Remove from system PATH
                $currentPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::Machine)
                if ($currentPath -like "*$BIN_DIR*") {
                    Write-Info "Removing $BIN_DIR from system PATH..."
                    $newPath = ($currentPath -split ';' | Where-Object { $_ -ne $BIN_DIR }) -join ';'
                    [Environment]::SetEnvironmentVariable("Path", $newPath, [EnvironmentVariableTarget]::Machine)
                    Write-Success "Removed from system PATH"
                }

                # Remove bin directory copy
                $shimPath = "$BIN_DIR\$APP_NAME.exe"
                if (Test-Path $shimPath) {
                    Remove-Item $shimPath -Force -ErrorAction SilentlyContinue
                    Write-Success "Bin copy removed"
                }
            }

            # Remove binary
            Write-Info "Removing binary..."
            Remove-Item $BIN_PATH -Force -ErrorAction SilentlyContinue
            Write-Success "Binary removed"

            # Remove installation directory if empty
            if (Test-Path $INSTALL_DIR) {
                $items = Get-ChildItem $INSTALL_DIR -ErrorAction SilentlyContinue
                if ($items.Count -eq 0) {
                    Remove-Item $INSTALL_DIR -Force
                    Write-Success "Installation directory removed"
                }
            }

            # Try to remove wenget home directory if empty
            if (Test-Path $WENGET_HOME) {
                $items = Get-ChildItem $WENGET_HOME -Recurse -ErrorAction SilentlyContinue
                if ($items.Count -eq 0) {
                    Remove-Item $WENGET_HOME -Force -Recurse
                    Write-Success "Wenget home directory removed"
                } else {
                    Write-Info "Wenget home directory contains other files, keeping it"
                }
            }
        }
    } else {
        Write-Info "Binary not found (already removed?)"

        # Still try to clean up system PATH for system-level
        if ($SYSTEM_INSTALL) {
            $currentPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::Machine)
            if ($currentPath -like "*$BIN_DIR*") {
                Write-Info "Removing $BIN_DIR from system PATH..."
                $newPath = ($currentPath -split ';' | Where-Object { $_ -ne $BIN_DIR }) -join ';'
                [Environment]::SetEnvironmentVariable("Path", $newPath, [EnvironmentVariableTarget]::Machine)
                Write-Success "Removed from system PATH"
            }
        }
    }

    Write-Info ""
    Write-Success "Uninstallation completed!"
    Write-Warning "Note: You may need to restart your terminal for PATH changes to take effect."
}

# Main
if ($Uninstall) {
    Uninstall-Wenget
} else {
    Install-Wenget
}
