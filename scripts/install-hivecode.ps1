#Requires -Version 5.1
<#
.SYNOPSIS
    HiveCode One-Command Installer for Windows
.DESCRIPTION
    Installs Rust, Node.js (if missing), Tauri CLI, frontend deps,
    builds HiveCode, and launches it.
.NOTES
    Copy and paste this ENTIRE block into PowerShell (Run as Administrator recommended):

    irm https://raw.githubusercontent.com/hivepowered/hivecode/main/scripts/install-hivecode.ps1 | iex

    OR if running locally:

    powershell -ExecutionPolicy Bypass -File scripts\install-hivecode.ps1
#>

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

function Write-Step($num, $text) { Write-Host "`n[$num] $text" -ForegroundColor Cyan }
function Write-Ok($text)         { Write-Host "  ✓ $text" -ForegroundColor Green }
function Write-Warn($text)       { Write-Host "  ! $text" -ForegroundColor Yellow }
function Write-Err($text)        { Write-Host "  ✗ $text" -ForegroundColor Red }

# ═══════════════════════════════════════════════════════════════
#  BANNER
# ═══════════════════════════════════════════════════════════════
Clear-Host
Write-Host ""
Write-Host "  ██╗  ██╗██╗██╗   ██╗███████╗ ██████╗ ██████╗ ██████╗ ███████╗" -ForegroundColor Yellow
Write-Host "  ██║  ██║██║██║   ██║██╔════╝██╔════╝██╔═══██╗██╔══██╗██╔════╝" -ForegroundColor Yellow
Write-Host "  ███████║██║██║   ██║█████╗  ██║     ██║   ██║██║  ██║█████╗  " -ForegroundColor Yellow
Write-Host "  ██╔══██║██║╚██╗ ██╔╝██╔══╝  ██║     ██║   ██║██║  ██║██╔══╝  " -ForegroundColor Yellow
Write-Host "  ██║  ██║██║ ╚████╔╝ ███████╗╚██████╗╚██████╔╝██████╔╝███████╗" -ForegroundColor Yellow
Write-Host "  ╚═╝  ╚═╝╚═╝  ╚═══╝  ╚══════╝ ╚═════╝ ╚═════╝ ╚═════╝ ╚══════╝" -ForegroundColor Yellow
Write-Host ""
Write-Host "  One-Command Setup & Build" -ForegroundColor White
Write-Host "  =========================" -ForegroundColor DarkGray
Write-Host ""

# ═══════════════════════════════════════════════════════════════
#  DETECT PROJECT PATH
# ═══════════════════════════════════════════════════════════════
$ProjectRoot = $null

# Check if we're already inside the project
if (Test-Path ".\Cargo.toml") {
    $content = Get-Content ".\Cargo.toml" -Raw
    if ($content -match "hivecode") { $ProjectRoot = (Get-Location).Path }
}

# Check common locations
if (-not $ProjectRoot) {
    $candidates = @(
        "$env:USERPROFILE\Documents\Claude\Projects\HiveCode\hivecode",
        "$env:USERPROFILE\Documents\HiveCode\hivecode",
        "$env:USERPROFILE\Desktop\HiveCode\hivecode",
        "$env:USERPROFILE\Projects\HiveCode\hivecode"
    )
    foreach ($c in $candidates) {
        if (Test-Path "$c\Cargo.toml") { $ProjectRoot = $c; break }
    }
}

if (-not $ProjectRoot) {
    Write-Err "Could not find HiveCode project. Please cd into the hivecode directory first."
    Write-Host "  Expected to find Cargo.toml in the current directory or common locations." -ForegroundColor DarkGray
    exit 1
}

Write-Host "  Project: $ProjectRoot" -ForegroundColor DarkGray
Set-Location $ProjectRoot

# ═══════════════════════════════════════════════════════════════
#  STEP 1: RUST
# ═══════════════════════════════════════════════════════════════
Write-Step 1 "Checking Rust toolchain..."

if (Get-Command rustc -ErrorAction SilentlyContinue) {
    Write-Ok "Rust $(rustc --version)"
} else {
    Write-Warn "Rust not found. Installing via rustup..."

    # Try winget first (cleanest), fall back to direct download
    if (Get-Command winget -ErrorAction SilentlyContinue) {
        winget install --id Rustlang.Rustup --accept-package-agreements --accept-source-agreements --silent
    } else {
        $rustupUrl = "https://win.rustup.rs/x86_64"
        $rustupExe = "$env:TEMP\rustup-init.exe"
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupExe
        Start-Process -FilePath $rustupExe -ArgumentList "-y" -Wait -NoNewWindow
        Remove-Item $rustupExe -Force
    }

    # Refresh PATH
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

    if (Get-Command rustc -ErrorAction SilentlyContinue) {
        Write-Ok "Rust installed: $(rustc --version)"
    } else {
        Write-Err "Rust installation failed. Please install manually: https://rustup.rs"
        exit 1
    }
}

# ═══════════════════════════════════════════════════════════════
#  STEP 2: NODE.JS
# ═══════════════════════════════════════════════════════════════
Write-Step 2 "Checking Node.js..."

if (Get-Command node -ErrorAction SilentlyContinue) {
    $nodeVer = (node --version).TrimStart("v").Split(".")[0]
    if ([int]$nodeVer -ge 18) {
        Write-Ok "Node.js $(node --version)"
    } else {
        Write-Warn "Node.js $(node --version) is too old (need 18+). Upgrading..."
        if (Get-Command winget -ErrorAction SilentlyContinue) {
            winget install --id OpenJS.NodeJS.LTS --accept-package-agreements --accept-source-agreements --silent
        }
    }
} else {
    Write-Warn "Node.js not found. Installing..."
    if (Get-Command winget -ErrorAction SilentlyContinue) {
        winget install --id OpenJS.NodeJS.LTS --accept-package-agreements --accept-source-agreements --silent
        # Refresh PATH
        $env:PATH = "$env:PROGRAMFILES\nodejs;$env:PATH"
    } else {
        Write-Err "Please install Node.js 18+ from https://nodejs.org"
        exit 1
    }

    if (Get-Command node -ErrorAction SilentlyContinue) {
        Write-Ok "Node.js installed: $(node --version)"
    } else {
        Write-Err "Node.js installation failed. Please install manually: https://nodejs.org"
        exit 1
    }
}

# ═══════════════════════════════════════════════════════════════
#  STEP 3: TAURI CLI
# ═══════════════════════════════════════════════════════════════
Write-Step 3 "Installing Tauri CLI..."

if (Get-Command cargo-tauri -ErrorAction SilentlyContinue) {
    Write-Ok "Tauri CLI already installed"
} else {
    cargo install tauri-cli --version "^2.0"
    if ($LASTEXITCODE -ne 0) {
        Write-Err "Tauri CLI install failed"
        exit 1
    }
    Write-Ok "Tauri CLI installed"
}

# ═══════════════════════════════════════════════════════════════
#  STEP 4: FRONTEND DEPS
# ═══════════════════════════════════════════════════════════════
Write-Step 4 "Installing frontend dependencies..."

Set-Location "$ProjectRoot\ui"
if (-not (Test-Path "node_modules\.package-lock.json")) {
    npm install --loglevel warn
    if ($LASTEXITCODE -ne 0) {
        Write-Err "npm install failed"
        exit 1
    }
}
Write-Ok "Frontend dependencies ready"
Set-Location $ProjectRoot

# ═══════════════════════════════════════════════════════════════
#  STEP 5: BUILD
# ═══════════════════════════════════════════════════════════════
Write-Step 5 "Building HiveCode (first build takes 3-5 minutes)..."

$buildStart = Get-Date
cargo tauri build 2>&1 | ForEach-Object {
    $line = $_
    if ($line -match "Compiling hivecode") { Write-Host "  ◈ $_" -ForegroundColor DarkCyan }
    elseif ($line -match "Finished|Bundling")  { Write-Host "  ◈ $_" -ForegroundColor Green }
    elseif ($line -match "error")              { Write-Host "  ◈ $_" -ForegroundColor Red }
}
$buildTime = ((Get-Date) - $buildStart).TotalSeconds

if ($LASTEXITCODE -ne 0) {
    Write-Err "Build failed! Check errors above."
    Write-Host ""
    Write-Host "  Common fixes:" -ForegroundColor Yellow
    Write-Host "    - Missing WebView2? Install from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/" -ForegroundColor DarkGray
    Write-Host "    - Cargo errors? Run: rustup update" -ForegroundColor DarkGray
    Write-Host "    - Node errors? Run: cd ui && npm install" -ForegroundColor DarkGray
    exit 1
}

# Find the built executable
$exe = Get-ChildItem "$ProjectRoot\target\release\hivecode.exe" -ErrorAction SilentlyContinue
if (-not $exe) {
    # Tauri might output to a different location
    $exe = Get-ChildItem "$ProjectRoot\target\release\bundle\nsis\*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
}

Write-Ok "Build complete in $([math]::Round($buildTime, 0))s"

if ($exe) {
    $sizeKB = [math]::Round($exe.Length / 1MB, 1)
    Write-Host ""
    Write-Host "  ╔══════════════════════════════════════════╗" -ForegroundColor Green
    Write-Host "  ║         HiveCode is ready!               ║" -ForegroundColor Green
    Write-Host "  ╠══════════════════════════════════════════╣" -ForegroundColor Green
    Write-Host "  ║  Binary: $($exe.FullName)" -ForegroundColor White
    Write-Host "  ║  Size:   $sizeKB MB" -ForegroundColor White
    Write-Host "  ╚══════════════════════════════════════════╝" -ForegroundColor Green
}

# ═══════════════════════════════════════════════════════════════
#  STEP 6: CREATE DEFAULT CONFIG
# ═══════════════════════════════════════════════════════════════
$configDir = "$env:USERPROFILE\.hivecode"
$configFile = "$configDir\config.toml"

if (-not (Test-Path $configFile)) {
    Write-Step 6 "Creating default configuration..."
    New-Item -ItemType Directory -Force -Path $configDir | Out-Null
    Copy-Item "$ProjectRoot\config.example.toml" $configFile
    Write-Ok "Config created at: $configFile"
    Write-Host "  Edit this file to add your API keys." -ForegroundColor DarkGray
} else {
    Write-Step 6 "Config already exists at $configFile"
}

# ═══════════════════════════════════════════════════════════════
#  LAUNCH
# ═══════════════════════════════════════════════════════════════
Write-Host ""
$launch = Read-Host "  Launch HiveCode now? (Y/n)"
if ($launch -ne "n" -and $launch -ne "N") {
    Write-Host "  Starting HiveCode..." -ForegroundColor Yellow
    if ($exe) {
        Start-Process -FilePath $exe.FullName
    } else {
        cargo tauri dev
    }
}

Write-Host ""
Write-Host "  Quick reference:" -ForegroundColor Cyan
Write-Host "    Dev mode:   cargo tauri dev" -ForegroundColor DarkGray
Write-Host "    Prod build: cargo tauri build" -ForegroundColor DarkGray
Write-Host "    Config:     $configFile" -ForegroundColor DarkGray
Write-Host ""
