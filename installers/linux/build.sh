#!/bin/bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# HiveCode ASCII Banner
print_banner() {
  echo -e "${CYAN}"
  cat << "EOF"
  ╔═══════════════════════════════════════╗
  ║                                       ║
  ║           ╔═══╗  ╔═══╗ ╔═╗ ╔═══╗     ║
  ║           ║ H ║  ║ I ║ ║V║ ║ E ║     ║
  ║           ╚═══╝  ╚═══╝ ╚═╝ ╚═══╝     ║
  ║                                       ║
  ║        AI-Powered Coding Assistant    ║
  ║                                       ║
  ╚═══════════════════════════════════════╝
EOF
  echo -e "${NC}"
}

# Print colored messages
print_info() {
  echo -e "${CYAN}[INFO]${NC} $1"
}

print_success() {
  echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
  echo -e "${RED}[ERROR]${NC} $1"
}

# Detect Linux distribution
detect_distro() {
  if [ -f /etc/os-release ]; then
    . /etc/os-release
    echo "${ID}"
  else
    echo "unknown"
  fi
}

# Check if command exists
command_exists() {
  command -v "$1" >/dev/null 2>&1
}

# Main build function
main() {
  print_banner

  # Detect distro
  DISTRO=$(detect_distro)
  print_info "Detected Linux distribution: $DISTRO"

  # Verify system dependencies
  print_info "Verifying system dependencies..."

  case "$DISTRO" in
    ubuntu|debian)
      DEPS_NEEDED=()

      if ! dpkg -l | grep -q webkit2gtk-4.1; then
        DEPS_NEEDED+=("webkit2gtk-4.1-dev")
      fi
      if ! dpkg -l | grep -q libgtk-3; then
        DEPS_NEEDED+=("libgtk-3-dev")
      fi
      if ! dpkg -l | grep -q libayatana-appindicator3; then
        DEPS_NEEDED+=("libayatana-appindicator3-dev")
      fi
      if ! dpkg -l | grep -q librsvg2; then
        DEPS_NEEDED+=("librsvg2-dev")
      fi
      if ! dpkg -l | grep -q pkg-config; then
        DEPS_NEEDED+=("pkg-config")
      fi
      if ! dpkg -l | grep -q libssl; then
        DEPS_NEEDED+=("libssl-dev")
      fi

      if [ ${#DEPS_NEEDED[@]} -gt 0 ]; then
        print_error "Missing system dependencies"
        echo -e "${CYAN}Install with:${NC}"
        echo "sudo apt-get install -y ${DEPS_NEEDED[*]}"
        exit 1
      fi
      print_success "All Ubuntu/Debian dependencies installed"
      ;;

    fedora|rhel|centos)
      DEPS_NEEDED=()

      if ! rpm -q webkit2gtk4.1-devel >/dev/null 2>&1; then
        DEPS_NEEDED+=("webkit2gtk4.1-devel")
      fi
      if ! rpm -q gtk3-devel >/dev/null 2>&1; then
        DEPS_NEEDED+=("gtk3-devel")
      fi
      if ! rpm -q libappindicator-gtk3-devel >/dev/null 2>&1; then
        DEPS_NEEDED+=("libappindicator-gtk3-devel")
      fi
      if ! rpm -q librsvg2-devel >/dev/null 2>&1; then
        DEPS_NEEDED+=("librsvg2-devel")
      fi
      if ! rpm -q pkg-config >/dev/null 2>&1; then
        DEPS_NEEDED+=("pkg-config")
      fi
      if ! rpm -q openssl-devel >/dev/null 2>&1; then
        DEPS_NEEDED+=("openssl-devel")
      fi

      if [ ${#DEPS_NEEDED[@]} -gt 0 ]; then
        print_error "Missing system dependencies"
        echo -e "${CYAN}Install with:${NC}"
        echo "sudo dnf install -y ${DEPS_NEEDED[*]}"
        exit 1
      fi
      print_success "All Fedora/RHEL dependencies installed"
      ;;

    arch|manjaro)
      DEPS_NEEDED=()

      if ! pacman -Q webkit2gtk-4.1 >/dev/null 2>&1; then
        DEPS_NEEDED+=("webkit2gtk-4.1")
      fi
      if ! pacman -Q gtk3 >/dev/null 2>&1; then
        DEPS_NEEDED+=("gtk3")
      fi
      if ! pacman -Q libayatana-appindicator >/dev/null 2>&1; then
        DEPS_NEEDED+=("libayatana-appindicator")
      fi
      if ! pacman -Q librsvg >/dev/null 2>&1; then
        DEPS_NEEDED+=("librsvg")
      fi
      if ! pacman -Q pkg-config >/dev/null 2>&1; then
        DEPS_NEEDED+=("pkg-config")
      fi
      if ! pacman -Q openssl >/dev/null 2>&1; then
        DEPS_NEEDED+=("openssl")
      fi

      if [ ${#DEPS_NEEDED[@]} -gt 0 ]; then
        print_error "Missing system dependencies"
        echo -e "${CYAN}Install with:${NC}"
        echo "sudo pacman -S ${DEPS_NEEDED[*]}"
        exit 1
      fi
      print_success "All Arch dependencies installed"
      ;;

    *)
      print_error "Unknown Linux distribution: $DISTRO"
      echo "Please ensure the following are installed:"
      echo "  - webkit2gtk-4.1-dev"
      echo "  - libgtk-3-dev"
      echo "  - libayatana-appindicator3-dev"
      echo "  - librsvg2-dev"
      echo "  - pkg-config"
      echo "  - libssl-dev"
      exit 1
      ;;
  esac

  # Verify development tools
  print_info "Verifying Rust and development tools..."

  if ! command_exists rustc; then
    print_error "Rust is not installed"
    echo "Install from: https://rustup.rs/"
    exit 1
  fi
  print_success "Rust $(rustc --version | cut -d' ' -f2) found"

  if ! command_exists node; then
    print_error "Node.js is not installed"
    echo "Install Node.js 18+ from: https://nodejs.org/"
    exit 1
  fi
  NODE_VERSION=$(node --version | cut -d'v' -f2 | cut -d'.' -f1)
  if [ "$NODE_VERSION" -lt 18 ]; then
    print_error "Node.js 18+ is required (found v$(node --version))"
    exit 1
  fi
  print_success "Node.js $(node --version) found"

  if ! command_exists cargo-tauri; then
    print_error "Tauri CLI is not installed"
    echo "Install with: cargo install tauri-cli"
    exit 1
  fi
  print_success "Tauri CLI found"

  # Build HiveCode
  print_info "Building HiveCode..."

  # Navigate to project root (two levels up from installers/linux)
  PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
  print_info "Project root: $PROJECT_ROOT"

  cd "$PROJECT_ROOT"

  # Install Node.js dependencies
  print_info "Installing Node.js dependencies..."
  if [ -d "ui" ]; then
    cd ui
    npm install
    cd ..
    print_success "Node.js dependencies installed"
  else
    print_error "ui/ directory not found in project root"
    exit 1
  fi

  # Run Tauri build
  print_info "Running Tauri build..."
  if cargo tauri build; then
    print_success "Tauri build completed"
  else
    print_error "Tauri build failed"
    exit 1
  fi

  # Create output directory if it doesn't exist
  mkdir -p installers/linux/output

  # Copy build artifacts
  print_info "Copying build artifacts..."

  # Find and copy .deb files
  if find src-tauri/target/release/bundle/deb -name "*.deb" 2>/dev/null | while read -r deb_file; do
    cp "$deb_file" installers/linux/output/
    print_success "Copied $(basename "$deb_file")"
  done | grep -q "Copied"; then
    :
  fi

  # Find and copy .AppImage files
  if find src-tauri/target/release/bundle/appimage -name "*.AppImage" 2>/dev/null | while read -r appimage_file; do
    cp "$appimage_file" installers/linux/output/
    chmod +x "installers/linux/output/$(basename "$appimage_file")"
    print_success "Copied and made executable $(basename "$appimage_file")"
  done | grep -q "Copied"; then
    :
  fi

  # Find and copy .rpm files if they exist
  if find src-tauri/target/release/bundle/rpm -name "*.rpm" 2>/dev/null | while read -r rpm_file; do
    cp "$rpm_file" installers/linux/output/
    print_success "Copied $(basename "$rpm_file")"
  done | grep -q "Copied"; then
    :
  fi

  # Generate checksums
  print_info "Generating SHA256 checksums..."
  cd installers/linux/output
  sha256sum * > CHECKSUMS 2>/dev/null || true

  # Report results
  echo ""
  print_success "Build completed successfully!"
  echo ""
  echo -e "${CYAN}Output files:${NC}"
  ls -lh | tail -n +2 | awk '{printf "  %-40s %s\n", $9, $5}'
  echo ""
  echo -e "${CYAN}Next steps:${NC}"
  echo "  1. Verify checksums: cd output && sha256sum -c CHECKSUMS"
  echo "  2. Install system-wide: sudo ../install-system.sh"
  echo "  3. Distribute the .deb, .rpm, or .AppImage file"
  echo ""
}

# Run main
main "$@"
