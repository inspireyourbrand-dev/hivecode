# HiveCode Installer Builder
# This script builds HiveCode and creates the Windows installer
#
# Prerequisites:
#   - Rust toolchain (rustc, cargo)
#   - Node.js 18+
#   - Inno Setup 6 (for .exe installer)
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts\build-installer.ps1

param(
    [switch]$SkipBuild,
    [switch]$SkipInstaller,
    [string]$InnoSetupPath = "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
)

$ErrorActionPreference = "Stop"
$ROOT = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

Write-Host ""
Write-Host "  ██╗  ██╗██╗██╗   ██╗███████╗ ██████╗ ██████╗ ██████╗ ███████╗" -ForegroundColor Yellow
Write-Host "  ██║  ██║██║██║   ██║██╔════╝██╔════╝██╔═══██╗██╔══██╗██╔════╝" -ForegroundColor Yellow
Write-Host "  ███████║██║██║   ██║█████╗  ██║     ██║   ██║██║  ██║█████╗  " -ForegroundColor Yellow
Write-Host "  ██╔══██║██║╚██╗ ██╔╝██╔══╝  ██║     ██║   ██║██║  ██║██╔══╝  " -ForegroundColor Yellow
Write-Host "  ██║  ██║██║ ╚████╔╝ ███████╗╚██████╗╚██████╔╝██████╔╝███████╗" -ForegroundColor Yellow
Write-Host "  ╚═╝  ╚═╝╚═╝  ╚═══╝  ╚══════╝ ╚═════╝ ╚═════╝ ╚═════╝ ╚══════╝" -ForegroundColor Yellow
Write-Host ""
Write-Host "  Build & Package System" -ForegroundColor Cyan
Write-Host "  ======================" -ForegroundColor DarkGray
Write-Host ""

Set-Location $ROOT

# ─── STEP 1: Verify toolchain ───
Write-Host "[1/5] Checking toolchain..." -ForegroundColor Cyan

$rustc = Get-Command rustc -ErrorAction SilentlyContinue
if (-not $rustc) { Write-Host "  ERROR: Rust not found. Run: winget install Rustlang.Rustup" -ForegroundColor Red; exit 1 }
Write-Host "  Rust: $(rustc --version)" -ForegroundColor Green

$node = Get-Command node -ErrorAction SilentlyContinue
if (-not $node) { Write-Host "  ERROR: Node.js not found. Run: winget install OpenJS.NodeJS.LTS" -ForegroundColor Red; exit 1 }
Write-Host "  Node: $(node --version)" -ForegroundColor Green

$cargo_tauri = Get-Command cargo-tauri -ErrorAction SilentlyContinue
if (-not $cargo_tauri) {
    Write-Host "  Installing Tauri CLI..." -ForegroundColor Yellow
    cargo install tauri-cli --version "^2.0"
}
Write-Host "  Tauri CLI: OK" -ForegroundColor Green

# ─── STEP 2: Install frontend deps ───
Write-Host ""
Write-Host "[2/5] Installing frontend dependencies..." -ForegroundColor Cyan
Set-Location "$ROOT\ui"
if (-not (Test-Path "node_modules")) {
    npm install
}
Write-Host "  Frontend deps: OK" -ForegroundColor Green
Set-Location $ROOT

# ─── STEP 3: Build with Tauri ───
if (-not $SkipBuild) {
    Write-Host ""
    Write-Host "[3/5] Building HiveCode (this may take a few minutes on first run)..." -ForegroundColor Cyan
    cargo tauri build 2>&1 | ForEach-Object {
        if ($_ -match "Compiling|Finished|Building|Bundling") {
            Write-Host "  $_" -ForegroundColor DarkGray
        }
    }

    if ($LASTEXITCODE -ne 0) {
        Write-Host "  ERROR: Build failed!" -ForegroundColor Red
        exit 1
    }
    Write-Host "  Build: OK" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "[3/5] Skipping build (--SkipBuild)" -ForegroundColor Yellow
}

# Verify the binary exists
$exePath = "$ROOT\target\release\hivecode.exe"
if (-not (Test-Path $exePath)) {
    Write-Host "  ERROR: $exePath not found. Run build first." -ForegroundColor Red
    exit 1
}
$size = (Get-Item $exePath).Length / 1MB
Write-Host "  Binary size: $([math]::Round($size, 1)) MB" -ForegroundColor Green

# ─── STEP 4: Download WebView2 bootstrapper ───
Write-Host ""
Write-Host "[4/5] Preparing installer assets..." -ForegroundColor Cyan

$webview2Path = "$ROOT\scripts\MicrosoftEdgeWebview2Setup.exe"
if (-not (Test-Path $webview2Path)) {
    Write-Host "  Downloading WebView2 bootstrapper..."
    Invoke-WebRequest -Uri "https://go.microsoft.com/fwlink/p/?LinkId=2124703" -OutFile $webview2Path
}
Write-Host "  WebView2 bootstrapper: OK" -ForegroundColor Green

# Create dist directory
New-Item -ItemType Directory -Force -Path "$ROOT\dist\installer" | Out-Null

# Create LICENSE if missing
if (-not (Test-Path "$ROOT\LICENSE")) {
    @"
MIT License

Copyright (c) 2026 HivePowered

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"@ | Out-File -FilePath "$ROOT\LICENSE" -Encoding UTF8
    Write-Host "  Created LICENSE file" -ForegroundColor Green
}

Write-Host "  Assets: OK" -ForegroundColor Green

# ─── STEP 5: Build installer ───
if (-not $SkipInstaller) {
    Write-Host ""
    Write-Host "[5/5] Building installer..." -ForegroundColor Cyan

    if (Test-Path $InnoSetupPath) {
        & $InnoSetupPath "$ROOT\scripts\installer.iss"
        if ($LASTEXITCODE -eq 0) {
            $installerPath = Get-ChildItem "$ROOT\dist\installer\*.exe" | Select-Object -First 1
            $installerSize = $installerPath.Length / 1MB
            Write-Host ""
            Write-Host "  ============================================" -ForegroundColor Green
            Write-Host "  Installer built successfully!" -ForegroundColor Green
            Write-Host "  Path: $($installerPath.FullName)" -ForegroundColor White
            Write-Host "  Size: $([math]::Round($installerSize, 1)) MB" -ForegroundColor White
            Write-Host "  ============================================" -ForegroundColor Green
        } else {
            Write-Host "  ERROR: Inno Setup failed." -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host "  Inno Setup not found at: $InnoSetupPath" -ForegroundColor Yellow
        Write-Host "  Falling back to portable ZIP..." -ForegroundColor Yellow

        # Create portable ZIP instead
        $zipPath = "$ROOT\dist\installer\HiveCode-0.1.0-portable.zip"
        Compress-Archive -Path $exePath, "$ROOT\config.example.toml" -DestinationPath $zipPath -Force
        Write-Host "  Portable ZIP: $zipPath" -ForegroundColor Green
    }
} else {
    Write-Host ""
    Write-Host "[5/5] Skipping installer (--SkipInstaller)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "  Done! " -ForegroundColor Green -NoNewline
Write-Host "HiveCode is ready to distribute." -ForegroundColor White
Write-Host ""
