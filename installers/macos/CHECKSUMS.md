# HiveCode macOS Installer Checksums

This file documents how to verify the integrity of HiveCode macOS installer artifacts.

## v0.1.0

After building HiveCode using `./build.sh`, verify the integrity of your build artifacts using SHA-256 checksums.

### Generate Checksums

Generate checksums for all files in the `output/` directory:

```bash
cd output/
shasum -a 256 * > CHECKSUMS.txt
cat CHECKSUMS.txt
```

### Example Output

```
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  HiveCode-0.1.0-x64.dmg
a1d0c6e83f027327d8461063f4ac58536e3d98a57afd2e66f2b6d8d0c2a3c4a5  HiveCode.app
```

### Verify Checksums

Store the checksums file and verify integrity later:

```bash
shasum -a 256 -c CHECKSUMS.txt
```

Expected output:
```
HiveCode-0.1.0-x64.dmg: OK
HiveCode.app: OK
```

### If Verification Fails

If checksums don't match:
1. Delete the `output/` directory
2. Run `cargo clean` in the repository root
3. Rebuild with `./build.sh`
4. Compare new checksums

Mismatched checksums could indicate:
- Filesystem corruption
- Partial build
- Modifications to the built files

## Code Signature Verification

### Verify Code Signature

If you built with code signing (`./build.sh --sign`), verify the signature:

```bash
codesign --verify --verbose ./output/HiveCode.app
spctl -a -t exec -vv ./output/HiveCode.app
```

Expected output:
```
./output/HiveCode.app: valid on disk
./output/HiveCode.app: accepted
```

### Verify Notarization

If the app was notarized, verify the notarization status:

```bash
spctl -a -t exec -vv ./output/HiveCode.app
```

A notarized app will show:
```
./output/HiveCode.app: accepted
source=Notarized Developer ID
```

## DMG Integrity

### Mount the DMG

```bash
hdiutil attach ./output/HiveCode-0.1.0-x64.dmg
```

The DMG should contain:
- `HiveCode.app/` - The application bundle
- `Applications` symlink (for drag-to-install)

### Verify DMG Checksum

```bash
shasum -a 256 ./output/HiveCode-0.1.0-x64.dmg
```

Compare with your stored checksum.

## Distribution Checklist

Before distributing HiveCode, verify:

- [ ] Checksums match expected values
- [ ] Code signature is valid: `codesign --verify`
- [ ] Notarization ticket is stapled (if applicable): `spctl -a -t exec -vv`
- [ ] DMG can be mounted without errors: `hdiutil attach`
- [ ] App runs without Gatekeeper warnings
- [ ] File permissions are correct: `ls -la output/`

## Troubleshooting

### "Code signature is invalid"

Rebuild with code signing:
```bash
./build.sh --sign "Developer ID Application: Your Name (TEAM_ID)"
```

### "Failed to verify" error

1. Ensure you're checking against the correct checksum
2. Verify the file hasn't been modified since building
3. Check filesystem for corruption: `diskutil verifyVolume /`

### Checksum Mismatch

Rebuild the artifacts:
```bash
rm -rf output/
cargo clean
./build.sh
```

Then regenerate checksums.

## Security Considerations

- Store checksums on a different system from the binaries
- Use a secure channel to distribute checksums
- For critical deployments, publish checksums in a trusted location (website, signed release notes, etc.)
- Consider using GPG signatures for additional verification

## References

- [Apple Code Signing Guide](https://developer.apple.com/support/)
- [macOS Notarization](https://developer.apple.com/documentation/macos/notarizing_macos_software_before_distribution)
- [shasum Manual](https://man.freebsd.org/cgi/man.cgi?query=shasum)
- [macOS Security Framework](https://developer.apple.com/documentation/security)
