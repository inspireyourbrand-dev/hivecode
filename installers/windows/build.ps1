# HiveCode Windows Installer Build Script
# This script builds the HiveCode desktop application and prepares the Windows installer

$ErrorActionPreference = "Stop"

# ASCII Banner
Write-Host @"
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║               HiveCode Windows Installer                  ║
║                   Build Automation                        ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
"@ -ForegroundColor Cyan

$startTime = Get-Date

# ============================================================================
# STEP 1: Verify Prerequisites
# ============================================================================
Write-Host "`n[1/6] Verifying prerequisites..." -ForegroundColor Yellow

try {
    $rustVersion = rustc --version 2>$null
    if (-not $rustVersion) {
        throw "Rust not installed"
    }
    Write-Host "  ✓ Rust: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "  ✗ Rust not found. Install from https://www.rust-lang.org/tools/install" -ForegroundColor Red
    exit 1
}

try {
    $nodeVersion = node --version 2>$null
    if (-not $nodeVersion) {
        throw "Node.js not installed"
    }
    Write-Host "  ✓ Node.js: $nodeVersion" -ForegroundColor Green
} catch {
    Write-Host "  ✗ Node.js not found. Install from https://nodejs.org/" -ForegroundColor Red
    exit 1
}

try {
    $npmVersion = npm --version 2>$null
    if (-not $npmVersion) {
        throw "npm not installed"
    }
    Write-Host "  ✓ npm: $npmVersion" -ForegroundColor Green
} catch {
    Write-Host "  ✗ npm not found" -ForegroundColor Red
    exit 1
}

try {
    cargo tauri --version 2>$null | Out-Null
    $tauriVersion = cargo tauri --version 2>&1
    Write-Host "  ✓ Tauri CLI: $tauriVersion" -ForegroundColor Green
} catch {
    Write-Host "  ! Tauri CLI not found globally, will use project-local installation" -ForegroundColor Yellow
}

# ============================================================================
# STEP 2: Navigate to HiveCode Root
# ============================================================================
Write-Host "`n[2/6] Setting up build directory..." -ForegroundColor Yellow

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$rootDir = Resolve-Path (Join-Path $scriptDir "..\..\")

if (-not (Test-Path (Join-Path $rootDir "Cargo.toml"))) {
    Write-Host "  ✗ Cannot find Cargo.toml at $rootDir" -ForegroundColor Red
    exit 1
}

Write-Host "  ✓ Root directory: $rootDir" -ForegroundColor Green
Push-Location $rootDir

# ============================================================================
# STEP 3: Install UI Dependencies
# ============================================================================
Write-Host "`n[3/6] Installing UI dependencies..." -ForegroundColor Yellow

$uiDir = Join-Path $rootDir "ui"
if (-not (Test-Path $uiDir)) {
    Write-Host "  ✗ UI directory not found at $uiDir" -ForegroundColor Red
    Pop-Location
    exit 1
}

Push-Location $uiDir
try {
    Write-Host "  Running: npm install" -ForegroundColor Gray
    npm install --legacy-peer-deps 2>&1 | ForEach-Object {
        if ($_ -match "^(npm ERR|npm WARN)") {
            Write-Host "  ⚠ $_" -ForegroundColor Yellow
        } elseif ($_ -match "added|up to date") {
            Write-Host "  ✓ $_" -ForegroundColor Green
        }
    }
    Write-Host "  ✓ UI dependencies installed" -ForegroundColor Green
} catch {
    Write-Host "  ✗ npm install failed: $_" -ForegroundColor Red
    Pop-Location
    Pop-Location
    exit 1
}
Pop-Location

# ============================================================================
# STEP 4: Build with Tauri
# ============================================================================
Write-Host "`n[4/6] Building HiveCode with Tauri..." -ForegroundColor Yellow

try {
    Write-Host "  Running: cargo tauri build" -ForegroundColor Gray
    $buildOutput = cargo tauri build 2>&1

    # Check for errors in output
    if ($buildOutput -match "error\[|error:") {
        Write-Host "  ✗ Build failed with errors:" -ForegroundColor Red
        $buildOutput | ForEach-Object {
            if ($_ -match "error") {
                Write-Host "  $_" -ForegroundColor Red
            }
        }
        Pop-Location
        exit 1
    }

    Write-Host "  ✓ Build completed successfully" -ForegroundColor Green
} catch {
    Write-Host "  ✗ Cargo tauri build failed: $_" -ForegroundColor Red
    Pop-Location
    exit 1
}

# ============================================================================
# STEP 5: Prepare Output Directory
# ============================================================================
Write-Host "`n[5/6] Preparing output directory..." -ForegroundColor Yellow

$outputDir = Join-Path $scriptDir "output"
$targetDir = Join-Path $rootDir "target\release\bundle\nsis"

# Create output directory if it doesn't exist
if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir | Out-Null
    Write-Host "  ✓ Created output directory" -ForegroundColor Green
}

# Copy NSIS installer
if (Test-Path $targetDir) {
    $nsissFiles = Get-ChildItem $targetDir -Filter "*.exe" -ErrorAction SilentlyContinue

    if ($nsissFiles) {
        foreach ($file in $nsissFiles) {
            $destPath = Join-Path $outputDir $file.Name
            Copy-Item $file.FullName $destPath -Force
            Write-Host "  ✓ Copied: $($file.Name)" -ForegroundColor Green
        }
    } else {
        Write-Host "  ⚠ No NSIS installer found in $targetDir" -ForegroundColor Yellow
    }
} else {
    Write-Host "  ⚠ Target directory not found at $targetDir" -ForegroundColor Yellow
}

# Also look for the built executable
$exePath = Join-Path $rootDir "target\release\hivecode.exe"
if (Test-Path $exePath) {
    $exeSize = (Get-Item $exePath).Length / 1MB
    Write-Host "  ✓ Built executable: $(Split-Path -Leaf $exePath) ($([math]::Round($exeSize, 2)) MB)" -ForegroundColor Green
} else {
    Write-Host "  ⚠ Executable not found at $exePath (may be named differently)" -ForegroundColor Yellow
}

# ============================================================================
# STEP 6: Report Results
# ============================================================================
Write-Host "`n[6/6] Build Summary" -ForegroundColor Yellow

if (Test-Path $outputDir) {
    $outputFiles = Get-ChildItem $outputDir -File -ErrorAction SilentlyContinue

    if ($outputFiles) {
        Write-Host "  Output files:" -ForegroundColor Green
        foreach ($file in $outputFiles) {
            $size = $file.Length / 1MB
            Write-Host "    • $($file.Name) ($([math]::Round($size, 2)) MB)" -ForegroundColor Green
        }
    } else {
        Write-Host "  No files in output directory" -ForegroundColor Yellow
    }
}

# Calculate and display build time
$endTime = Get-Date
$buildTime = $endTime - $startTime
Write-Host "`n  Build completed in: $([math]::Round($buildTime.TotalMinutes, 2)) minutes" -ForegroundColor Cyan

# ============================================================================
# Completion
# ============================================================================
Pop-Location

Write-Host "`n╔═══════════════════════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "║                                                           ║" -ForegroundColor Green
Write-Host "║  ✓ Build successful! Installers ready in:                ║" -ForegroundColor Green
Write-Host "║    .\output\                                              ║" -ForegroundColor Green
Write-Host "║                                                           ║" -ForegroundColor Green
Write-Host "║  Next steps:                                              ║" -ForegroundColor Green
Write-Host "║  1. Verify checksums (see CHECKSUMS.md)                  ║" -ForegroundColor Green
Write-Host "║  2. Test installer on Windows 10/11                      ║" -ForegroundColor Green
Write-Host "║  3. (Optional) Build custom Inno Setup installer         ║" -ForegroundColor Green
Write-Host "║     by opening installer.iss in Inno Setup 6            ║" -ForegroundColor Green
Write-Host "║                                                           ║" -ForegroundColor Green
Write-Host "╚═══════════════════════════════════════════════════════════╝" -ForegroundColor Green
