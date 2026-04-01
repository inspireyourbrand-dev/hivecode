#!/bin/bash
# HiveCode Setup Script for macOS/Linux
# Run: chmod +x scripts/setup.sh && ./scripts/setup.sh

set -e

echo "============================================"
echo "  HiveCode Development Environment Setup"
echo "============================================"
echo ""

# Check for Rust
if command -v rustc &> /dev/null; then
    echo "[OK] Rust found: $(rustc --version)"
else
    echo "[!] Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "[OK] Rust installed."
fi

# Check for Node.js
if command -v node &> /dev/null; then
    echo "[OK] Node.js found: $(node --version)"
else
    echo "[!] Node.js not found. Please install Node.js 18+ from https://nodejs.org"
    exit 1
fi

# Install system dependencies (Linux only)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo ""
    echo "Installing system dependencies (requires sudo)..."
    sudo apt-get update -qq
    sudo apt-get install -y -qq \
        libwebkit2gtk-4.1-dev \
        libgtk-3-dev \
        libayatana-appindicator3-dev \
        librsvg2-dev \
        pkg-config \
        libssl-dev
    echo "[OK] System dependencies installed."
fi

# Install Tauri CLI
echo ""
echo "Installing Tauri CLI..."
cargo install tauri-cli --version "^2.0"
echo "[OK] Tauri CLI installed."

# Install frontend dependencies
echo ""
echo "Installing frontend dependencies..."
cd ui
npm install
cd ..
echo "[OK] Frontend dependencies installed."

echo ""
echo "============================================"
echo "  Setup Complete!"
echo "============================================"
echo ""
echo "To start development:"
echo "  cargo tauri dev"
echo ""
echo "To build for production:"
echo "  cargo tauri build"
echo ""
