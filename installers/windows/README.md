# HiveCode Windows Installer Package

## Overview

This folder contains everything needed to build a Windows installer for **HiveCode**, a Rust + Tauri v2 desktop application. The build system produces two installer options:

1. **NSIS Installer** (via Tauri): `HiveCode-0.1.0-x64-setup.exe` (~12 MB)
2. **Inno Setup Installer** (optional): `HiveCode-0.1.0-Setup.exe` (~10 MB)

Both installers support Windows 10/11, include automatic WebView2 runtime installation, and register hivecode:// URL protocol support.

## Prerequisites

Before building, ensure you have installed:

- **Rust 1.70+** ([https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install))
- **Node.js 18+** ([https://nodejs.org/](https://nodejs.org/))
  - Verify: `node --version` and `npm --version`
- **Tauri CLI**: Installed via npm in the project (or globally with `cargo install tauri-cli`)
- **Inno Setup 6** (optional, for custom installer): [https://jrsoftware.org/isinfo.php](https://jrsoftware.org/isinfo.php)

Verify installation:
```powershell
rustc --version
cargo --version
node --version
npm --version
```

## Building

### Quick Build (NSIS via Tauri)

Run the PowerShell build script from this directory:

```powershell
.\build.ps1
```

This script:
1. Verifies all prerequisites
2. Navigates to the HiveCode root directory
3. Installs Node.js dependencies (`npm install` in `ui/`)
4. Builds the Rust application and Tauri installer with `cargo tauri build`
5. Copies artifacts to `.\output/`
6. Reports file sizes and success

Expected build time: 5-15 minutes (depending on your system and whether this is the first build).

### Custom Build (Inno Setup)

After running `build.ps1`, you can create a polished Inno Setup installer:

1. Install Inno Setup 6 from [https://jrsoftware.org/isinfo.php](https://jrsoftware.org/isinfo.php)
2. Open `installer.iss` with Inno Setup IDE
3. Click **Build** → **Compile**, or press `Ctrl+F9`
4. The compiled installer will be saved to `.\output\HiveCode-0.1.0-Setup.exe`

Alternatively, compile from PowerShell if Inno Setup is in PATH:
```powershell
& "C:\Program Files (x86)\Inno Setup 6\Compil32.exe" /cc installer.iss
```

## Output

Both build processes produce installers in the `output/` subdirectory:

| Installer | Filename | Size | Source |
|-----------|----------|------|--------|
| NSIS (Tauri) | `HiveCode-0.1.0-x64-setup.exe` | ~12 MB | `../../target/release/` |
| Inno Setup | `HiveCode-0.1.0-Setup.exe` | ~10 MB | `installer.iss` |

### Installation Details

- **Default install location**: `C:\Program Files\HiveCode` (or `C:\Program Files (x86)\HiveCode` on 32-bit systems)
- **Start Menu shortcuts**: Created automatically
- **URL protocol**: `hivecode://` registered for deep linking
- **WebView2 Runtime**: Downloaded and installed automatically if not present
- **Uninstaller**: Full uninstall available via Control Panel > Programs > Programs and Features

## File Structure

```
installers/windows/
├── README.md                    # This file
├── build.ps1                    # PowerShell build automation script
├── installer.iss                # Inno Setup 6 configuration script
├── CHECKSUMS.md                 # Checksum verification guide
└── output/                      # Generated after build (not in repo)
    ├── HiveCode-0.1.0-x64-setup.exe
    └── HiveCode-0.1.0-Setup.exe (if using Inno Setup)
```

## Troubleshooting

### Build fails: "Rust not found"
- Install Rust: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
- Restart your PowerShell terminal

### Build fails: "Node.js not found"
- Install Node.js: [https://nodejs.org/](https://nodejs.org/)
- Verify: `node --version`
- Restart PowerShell

### Build fails: "tauri command not found"
- Install Tauri CLI globally: `cargo install tauri-cli`
- Or ensure project dependencies are installed: `cd ../../ && npm install`

### Inno Setup script fails
- Verify paths in `installer.iss` match your actual build output
- Check that `../../target/release/hivecode.exe` exists
- Ensure Inno Setup 6 is installed (not Inno Setup 5)

## Notes

- All builds produce **64-bit installers** targeting Windows 10+
- WebView2 is required; the installer handles this automatically
- License file is sourced from the repository root: `../../LICENSE`
- Application icon is sourced from: `../../crates/hivecode-tauri/icons/icon.ico`
- Build artifacts are cached; use `cargo clean` if you need a full rebuild

## Support

For questions or issues with the build process:
1. Check this README and the CHECKSUMS.md file
2. Review the PowerShell output for specific error messages
3. Verify all prerequisites are correctly installed
4. Consult the Tauri docs: [https://tauri.app/](https://tauri.app/)
