# HiveCode macOS Installer Package

## Overview

This folder contains everything needed to build a macOS installer for **HiveCode**, a Rust + Tauri v2 desktop application. The build system produces a native macOS app bundle and a distributable DMG file:

1. **HiveCode.app** - Native macOS application bundle
2. **HiveCode-0.1.0-x64.dmg** - Disk image for distribution (~25 MB)

Both outputs support macOS 10.15+ and include native code signing and notarization support.

## Prerequisites

Before building, ensure you have installed:

- **Xcode Command Line Tools**
  - Install: `xcode-select --install`
  - Verify: `xcode-select --version`
- **Rust 1.70+** ([https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install))
  - Verify: `rustc --version` and `cargo --version`
- **Node.js 18+** ([https://nodejs.org/](https://nodejs.org/))
  - Verify: `node --version` and `npm --version`
- **Tauri CLI** (installed via cargo)
  - Install: `cargo install tauri-cli`
  - Verify: `cargo tauri --version`

Verify all prerequisites:
```bash
xcode-select --version
rustc --version
cargo --version
node --version
npm --version
cargo tauri --version
```

## Building

### Quick Build

Run the build script from this directory:

```bash
./build.sh
```

This script will:
1. Verify all prerequisites are installed
2. Navigate to the HiveCode root directory
3. Install Node.js dependencies (`npm install` in `ui/`)
4. Build the Rust application and Tauri app bundle with `cargo tauri build`
5. Copy artifacts to `./output/`
6. Report file sizes and build success

Expected build time: 5-15 minutes (depending on your system and whether this is the first build).

### Build with Code Signing

If you have a valid macOS Developer ID certificate, you can automatically code sign the app:

```bash
./build.sh --sign "Developer ID Application: Your Name (TEAM_ID)"
```

Or use the default Xcode identity:

```bash
./build.sh --sign -
```

The script will:
1. Build the app normally
2. Code sign the HiveCode.app bundle with your Developer ID
3. Create the DMG
4. Optionally notarize the app (see notarization section below)

### Build Output

After a successful build, the `./output/` directory will contain:

| File | Size | Description |
|------|------|-------------|
| `HiveCode-0.1.0-x64.dmg` | ~25 MB | Distributable disk image |
| `HiveCode.app/` | ~80 MB | Signed application bundle (inside DMG) |

The DMG can be distributed to users, who can mount it and drag HiveCode.app to their Applications folder.

## Code Signing

### Why Sign Your App?

- **Gatekeeper**: macOS verifies the signature before running
- **Distribution**: Code signing is required for Mac App Store or direct distribution
- **User Trust**: Users see your app is verified and not tampered with

### Prerequisites for Signing

1. **Apple Developer Account** ([https://developer.apple.com/](https://developer.apple.com/))
2. **Developer ID Certificate** - Create one in your Apple Developer account
3. **Keychain Setup** - Import your certificate into your Keychain (done automatically when downloading)

### Signing Your Build

The `build.sh` script supports the `--sign` flag:

```bash
./build.sh --sign "Developer ID Application: Your Name (TEAM_ID)"
```

Or to use the default Xcode identity:

```bash
./build.sh --sign -
```

You can view available signing identities:

```bash
security find-identity -v -p codesigning
```

### Manual Signing (Advanced)

If you prefer to sign manually after building:

```bash
codesign -s "Developer ID Application: Your Name (TEAM_ID)" \
  --force \
  --deep \
  --timestamp \
  --options=runtime \
  --entitlements ./entitlements.plist \
  ./output/HiveCode.app
```

## Notarization

### What is Notarization?

Notarization is Apple's security service that verifies your app doesn't contain malware. Starting with macOS 10.15 Catalina, notarized apps run without Gatekeeper warnings.

### Notarization Requirements

- Your app must be **code signed** with a Developer ID
- Apple Developer Account with notarization enabled
- App must be packaged in a **signed ZIP or DMG**

### Notarizing Your App

After code signing, create a notarization package:

```bash
ditto -c -k --sequesterRsrc ./output/HiveCode.app ./output/HiveCode.zip
```

Then submit for notarization:

```bash
xcrun notarytool submit ./output/HiveCode.zip \
  --apple-id "your-apple-id@example.com" \
  --password "app-specific-password" \
  --team-id "TEAM_ID" \
  --wait
```

Check notarization status:

```bash
xcrun notarytool info <submission-id> \
  --apple-id "your-apple-id@example.com" \
  --password "app-specific-password" \
  --team-id "TEAM_ID"
```

Once notarized, staple the ticket to your app:

```bash
xcrun stapler staple ./output/HiveCode.app
```

### Automated Notarization (macOS 12+)

For newer macOS versions, use the `--notarize` flag (requires `xcrun` 12.0+):

```bash
./build.sh --sign "Developer ID Application: Your Name" --notarize
```

Note: This requires storing Apple credentials securely (preferably in Keychain or environment variables).

## File Structure

```
installers/macos/
├── README.md                    # This file
├── build.sh                     # Bash build automation script
├── entitlements.plist          # macOS entitlements configuration
├── Info.plist.override         # Supplementary app info
├── CHECKSUMS.md                # Checksum verification guide
└── output/                      # Generated after build (not in repo)
    ├── HiveCode-0.1.0-x64.dmg
    └── HiveCode.app/
```

## Troubleshooting

### Build fails: "Rust not found"
- Install Rust: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
- Restart your terminal

### Build fails: "Xcode CLI tools not found"
- Install Xcode Command Line Tools: `xcode-select --install`
- Accept Xcode license: `sudo xcode-select --reset`
- Restart your terminal

### Build fails: "Node.js not found"
- Install Node.js: [https://nodejs.org/](https://nodejs.org/)
- Verify: `node --version`
- Restart terminal

### Build fails: "tauri command not found"
- Install Tauri CLI: `cargo install tauri-cli`
- Verify: `cargo tauri --version`

### Code signing fails: "identity not found"
- List available identities: `security find-identity -v -p codesigning`
- Ensure you're using the correct identity name
- Check that your certificate is in Keychain: Keychain Access app > Certificates

### "The code signature is invalid" error
- Ensure the entitlements.plist file is present and valid
- Re-sign with `--force` flag
- Check file permissions: ensure app bundle is readable

### Notarization fails: "Invalid toolchain"
- Ensure `xcrun` is available: `xcrun --version`
- Update Xcode Command Line Tools: `sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer`

## Build Environment

- **Target**: Apple Silicon (arm64) and Intel (x86_64) - universal builds supported
- **Minimum macOS Version**: 10.15 Catalina
- **Frameworks**: Uses native Tauri runtime, WebKit for web frontend
- **Dependencies**: All Rust dependencies are managed by Cargo; Node.js dependencies by npm

## Notes

- The build script uses `set -euo pipefail` for safety - it will exit on any error
- Builds are incremental; use `cargo clean` in the root directory for a full rebuild
- The app bundle is code-signed automatically when using `--sign` flag
- DMG creation is handled by Tauri's bundler
- License file is sourced from the repository root: `../../LICENSE`
- Application icons are sourced from: `../../crates/hivecode-tauri/icons/`

## Support

For questions or issues with the build process:
1. Check this README and the CHECKSUMS.md file
2. Review the build script output for specific error messages
3. Verify all prerequisites are correctly installed
4. Consult the Tauri docs: [https://tauri.app/](https://tauri.app/)
5. Consult Apple's code signing docs: [https://developer.apple.com/support/](https://developer.apple.com/support/)
