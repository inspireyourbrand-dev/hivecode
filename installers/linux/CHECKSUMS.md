# HiveCode Build Checksums

This file documents how to verify the integrity of HiveCode distribution packages.

## Verify SHA256 Checksums

After building HiveCode, verify the integrity of the output files using SHA256:

```bash
cd output
sha256sum -c CHECKSUMS
```

All files should show as "OK":

```
hivecode-X.X.X_amd64.deb: OK
hivecode-X.X.X.AppImage: OK
CHECKSUMS: OK
```

## Manual Verification

If you want to manually generate and verify checksums:

```bash
# Generate checksums for all files in output directory
cd output
sha256sum * > CHECKSUMS

# Verify all files match their checksums
sha256sum -c CHECKSUMS

# Verify a single file
sha256sum hivecode-X.X.X_amd64.deb
```

## Distribution Verification

Users can verify downloaded files before installation:

```bash
# After downloading the .deb package
sha256sum ./hivecode-X.X.X_amd64.deb

# Compare the output with the published checksum
```

## Security Best Practices

1. **Verify before installation** — Always check the checksum before installing
2. **Use official sources** — Download from official HiveCode repositories only
3. **Compare publicly** — Checksums should be published on a secure, trusted channel
4. **Keep records** — Save checksums for version tracking

## Checksum File Format

The `CHECKSUMS` file in the output directory contains lines in the format:

```
<hash>  <filename>
```

Example:

```
a1b2c3d4e5f6... hivecode-1.0.0_amd64.deb
f6e5d4c3b2a1... hivecode-1.0.0.AppImage
```

Each checksum verifies the integrity of its corresponding file.
