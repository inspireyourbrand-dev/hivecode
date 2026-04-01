#!/bin/bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

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

print_warning() {
  echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check if running as root
check_root() {
  if [ "$EUID" -ne 0 ]; then
    print_error "This script must be run with sudo"
    echo "Usage: sudo ./install-system.sh [--uninstall]"
    exit 1
  fi
}

# Find the HiveCode binary in the output directory
find_hivecode_binary() {
  local script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  local output_dir="$script_dir/output"

  if [ ! -d "$output_dir" ]; then
    print_error "Output directory not found: $output_dir"
    echo "Please run ./build.sh first to build HiveCode"
    exit 1
  fi

  # Try to find the AppImage or binary
  local appimage_file=$(find "$output_dir" -maxdepth 1 -name "*.AppImage" | head -n 1)
  if [ -z "$appimage_file" ]; then
    print_error "No .AppImage file found in $output_dir"
    echo "Please run ./build.sh first to build HiveCode"
    exit 1
  fi

  echo "$appimage_file"
}

# Install HiveCode system-wide
install_hivecode() {
  print_info "Installing HiveCode system-wide..."

  local script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  local binary_file="$(find_hivecode_binary)"
  local icon_dir="$script_dir"
  local desktop_file="$script_dir/hivecode.desktop"

  # Verify files exist
  if [ ! -f "$binary_file" ]; then
    print_error "Binary file not found: $binary_file"
    exit 1
  fi

  if [ ! -f "$desktop_file" ]; then
    print_error "Desktop file not found: $desktop_file"
    exit 1
  fi

  # Copy binary to /usr/local/bin
  print_info "Copying HiveCode binary to /usr/local/bin/..."
  cp "$binary_file" /usr/local/bin/hivecode
  chmod +x /usr/local/bin/hivecode
  print_success "Binary installed: /usr/local/bin/hivecode"

  # Copy desktop file to /usr/share/applications
  print_info "Installing desktop entry..."
  cp "$desktop_file" /usr/share/applications/hivecode.desktop
  print_success "Desktop entry installed: /usr/share/applications/hivecode.desktop"

  # Create icon directory if needed
  mkdir -p /usr/share/icons/hicolor/256x256/apps

  # Look for icon file in the script directory
  if [ -f "$icon_dir/hivecode.png" ]; then
    print_info "Installing application icon..."
    cp "$icon_dir/hivecode.png" /usr/share/icons/hicolor/256x256/apps/hivecode.png
    print_success "Icon installed: /usr/share/icons/hicolor/256x256/apps/hivecode.png"
  else
    print_warning "No hivecode.png found in $icon_dir"
    print_warning "Consider adding a 256x256 PNG icon for better integration"
  fi

  # Register hivecode:// URL scheme
  print_info "Registering hivecode:// URL scheme..."
  xdg-mime default hivecode.desktop x-scheme-handler/hivecode
  print_success "URL scheme registered"

  # Update desktop database
  print_info "Updating desktop database..."
  update-desktop-database /usr/share/applications 2>/dev/null || true
  print_success "Desktop database updated"

  # Update icon cache
  print_info "Updating icon cache..."
  gtk-update-icon-cache /usr/share/icons/hicolor 2>/dev/null || true
  print_success "Icon cache updated"

  echo ""
  print_success "HiveCode installed successfully!"
  echo ""
  echo -e "${CYAN}Installation details:${NC}"
  echo "  Binary:       /usr/local/bin/hivecode"
  echo "  Desktop:      /usr/share/applications/hivecode.desktop"
  echo "  Icon:         /usr/share/icons/hicolor/256x256/apps/hivecode.png"
  echo "  URL Scheme:   hivecode://"
  echo ""
  echo -e "${CYAN}You can now:${NC}"
  echo "  1. Launch from application menu (HiveCode)"
  echo "  2. Run from terminal: hivecode"
  echo "  3. Open hivecode:// URLs from web browsers"
  echo ""
  echo -e "${CYAN}To uninstall:${NC}"
  echo "  sudo ./install-system.sh --uninstall"
  echo ""
}

# Uninstall HiveCode
uninstall_hivecode() {
  print_info "Uninstalling HiveCode..."

  local files_removed=0

  # Remove binary
  if [ -f /usr/local/bin/hivecode ]; then
    rm -f /usr/local/bin/hivecode
    print_success "Removed: /usr/local/bin/hivecode"
    ((files_removed++))
  fi

  # Remove desktop file
  if [ -f /usr/share/applications/hivecode.desktop ]; then
    rm -f /usr/share/applications/hivecode.desktop
    print_success "Removed: /usr/share/applications/hivecode.desktop"
    ((files_removed++))
  fi

  # Remove icon
  if [ -f /usr/share/icons/hicolor/256x256/apps/hivecode.png ]; then
    rm -f /usr/share/icons/hicolor/256x256/apps/hivecode.png
    print_success "Removed: /usr/share/icons/hicolor/256x256/apps/hivecode.png"
    ((files_removed++))
  fi

  # Update desktop database
  print_info "Updating desktop database..."
  update-desktop-database /usr/share/applications 2>/dev/null || true

  # Update icon cache
  print_info "Updating icon cache..."
  gtk-update-icon-cache /usr/share/icons/hicolor 2>/dev/null || true

  echo ""
  if [ "$files_removed" -gt 0 ]; then
    print_success "HiveCode uninstalled successfully!"
    echo "Removed $files_removed file(s)"
  else
    print_warning "HiveCode does not appear to be installed"
  fi
  echo ""
}

# Main
main() {
  check_root

  case "${1:-}" in
    --uninstall|-u|uninstall)
      uninstall_hivecode
      ;;
    *)
      install_hivecode
      ;;
  esac
}

# Run main
main "$@"
