# HiveCode Linux Installer Package

This folder contains everything needed to build and distribute HiveCode on Linux.

## Contents

- **build.sh** — Main build script that detects distro, verifies dependencies, and builds HiveCode
- **hivecode.desktop** — Linux desktop entry file for application launchers
- **install-system.sh** — Post-build system installation script (requires sudo)
- **CHECKSUMS.md** — SHA256 verification guide
- **README.md** — This file

## Prerequisites

### Rust & Build Tools
- Rust 1.70+ and Cargo (install via [rustup.rs](https://rustup.rs))
- Node.js 18+ and npm

### Tauri CLI
```bash
cargo install tauri-cli
```

### System Dependencies

**Ubuntu/Debian:**
```bash
sudo apt-get install -y \
  webkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  pkg-config \
  libssl-dev
```

**Fedora/RHEL:**
```bash
sudo dnf install -y \
  webkit2gtk4.1-devel \
  gtk3-devel \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  pkg-config \
  openssl-devel
```

**Arch Linux:**
```bash
sudo pacman -S \
  webkit2gtk-4.1 \
  gtk3 \
  libayatana-appindicator \
  librsvg \
  pkg-config \
  openssl
```

## Building HiveCode

### Step 1: Verify Prerequisites
The `build.sh` script automatically detects your distro and verifies all dependencies. If any are missing, it will display the installation command.

### Step 2: Run the Build Script
```bash
cd /path/to/hivecode/installers/linux
chmod +x build.sh
./build.sh
```

The script will:
1. Display the HiveCode banner
2. Detect your Linux distribution
3. Verify all system dependencies are installed
4. Verify Rust, Node.js, and Tauri CLI
5. Install Node.js dependencies (`npm install` in ui/)
6. Build the application (`cargo tauri build`)
7. Copy the generated .deb and .AppImage files to `output/`
8. Report success with file sizes

### Step 3: Verify Checksums (Optional)
```bash
cd output
sha256sum -c CHECKSUMS
```

### Step 4: Install System-Wide (Optional)
```bash
sudo ./install-system.sh
```

This will:
- Copy the binary to `/usr/local/bin/hivecode`
- Install the .desktop file for launchers
- Register the `hivecode://` URL scheme
- Update the desktop database

## Build Output

The `output/` directory will contain:

- **hivecode-X.X.X_amd64.deb** — Debian/Ubuntu package
- **hivecode-X.X.X-1.x86_64.rpm** — Fedora/RHEL package (if building on those distros)
- **hivecode-X.X.X.AppImage** — Universal Linux AppImage
- **CHECKSUMS** — SHA256 checksums for verification

## Distribution Notes

### Ubuntu/Debian
The .deb package is the recommended distribution method. Users can install with:
```bash
sudo dpkg -i hivecode-X.X.X_amd64.deb
```

### Fedora/RHEL
Distribute the .rpm package. Install with:
```bash
sudo dnf install hivecode-X.X.X-1.x86_64.rpm
```

### Arch Linux
Consider submitting to AUR, or distribute the .AppImage. Users install with:
```bash
chmod +x hivecode-X.X.X.AppImage
./hivecode-X.X.X.AppImage
```

### Universal (All Distros)
The .AppImage is fully portable. No installation needed—just make executable and run. Recommended for users who prefer not to install system-wide.

## Uninstalling

If you used `install-system.sh` to install system-wide:
```bash
sudo ./install-system.sh --uninstall
```

## Troubleshooting

**Missing dependencies error?**
The `build.sh` script will display the exact `apt-get`, `dnf`, or `pacman` command needed.

**Rust not found?**
Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

**Node.js version too old?**
Install Node.js 18+: https://nodejs.org/

**Tauri CLI not found?**
Install it: `cargo install tauri-cli`

## Support

For issues, questions, or contributions, visit the HiveCode repository.
