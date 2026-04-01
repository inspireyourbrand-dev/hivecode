# HiveCode Setup Script for Windows
# Run: powershell -ExecutionPolicy Bypass -File scripts/setup.ps1

Write-Host "============================================" -ForegroundColor Yellow
Write-Host "  HiveCode Development Environment Setup" -ForegroundColor Yellow
Write-Host "============================================" -ForegroundColor Yellow
Write-Host ""

# Check for Rust
$rustc = Get-Command rustc -ErrorAction SilentlyContinue
if (-not $rustc) {
    Write-Host "[!] Rust not found. Installing via rustup..." -ForegroundColor Red
    Write-Host "    Downloading rustup-init.exe..."
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
    & "$env:TEMP\rustup-init.exe" -y
    $env:PATH += ";$env:USERPROFILE\.cargo\bin"
    Write-Host "[OK] Rust installed." -ForegroundColor Green
} else {
    Write-Host "[OK] Rust found: $(rustc --version)" -ForegroundColor Green
}

# Check for Node.js
$node = Get-Command node -ErrorAction SilentlyContinue
if (-not $node) {
    Write-Host "[!] Node.js not found. Please install Node.js 18+ from https://nodejs.org" -ForegroundColor Red
    exit 1
} else {
    Write-Host "[OK] Node.js found: $(node --version)" -ForegroundColor Green
}

# Install Tauri CLI
Write-Host ""
Write-Host "Installing Tauri CLI..." -ForegroundColor Cyan
cargo install tauri-cli --version "^2.0"
Write-Host "[OK] Tauri CLI installed." -ForegroundColor Green

# Install frontend dependencies
Write-Host ""
Write-Host "Installing frontend dependencies..." -ForegroundColor Cyan
Set-Location -Path "ui"
npm install
Set-Location -Path ".."
Write-Host "[OK] Frontend dependencies installed." -ForegroundColor Green

# Check for WebView2 (Windows requirement for Tauri)
$webview2 = Get-ItemProperty -Path "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" -ErrorAction SilentlyContinue
if (-not $webview2) {
    Write-Host "[!] WebView2 Runtime not detected." -ForegroundColor Yellow
    Write-Host "    Tauri requires WebView2. It's included with Windows 10 (Nov 2021+) and Windows 11." -ForegroundColor Yellow
    Write-Host "    Download: https://developer.microsoft.com/en-us/microsoft-edge/webview2/" -ForegroundColor Yellow
} else {
    Write-Host "[OK] WebView2 Runtime detected." -ForegroundColor Green
}

Write-Host ""
Write-Host "============================================" -ForegroundColor Yellow
Write-Host "  Setup Complete!" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Yellow
Write-Host ""
Write-Host "To start development:" -ForegroundColor Cyan
Write-Host "  cargo tauri dev" -ForegroundColor White
Write-Host ""
Write-Host "To build for production:" -ForegroundColor Cyan
Write-Host "  cargo tauri build" -ForegroundColor White
Write-Host ""
