# HiveCode Windows Installer Checksums

Verify the integrity and authenticity of HiveCode installers using cryptographic checksums.

## v0.1.0

After building the installers with `build.ps1`, generate and verify checksums as follows.

### Generating Checksums

Run this PowerShell command in the `output/` directory to generate SHA256 checksums:

```powershell
Get-FileHash .\*.exe -Algorithm SHA256 | Format-Table -AutoSize
```

Or to save checksums to a file for distribution:

```powershell
Get-FileHash .\*.exe -Algorithm SHA256 | Export-Csv .\CHECKSUMS_SHA256.csv -NoTypeInformation
```

For a human-readable text file:

```powershell
Get-FileHash .\*.exe -Algorithm SHA256 | ForEach-Object { "$($_.Hash)  $($_.Path)" } | Out-File CHECKSUMS.txt
```

### Example Output

After building, you should see output similar to this:

```
Algorithm       Hash                                                   Path
---------       ----                                                   ----
SHA256          A1B2C3D4E5F6... (64 characters)                       .\HiveCode-0.1.0-x64-setup.exe
SHA256          F6E5D4C3B2A1... (64 characters)                       .\HiveCode-0.1.0-Setup.exe
```

### Verifying Downloaded Installers

To verify an installer you've downloaded:

1. Open PowerShell in the directory containing the installer
2. Run the checksum command above
3. Compare the hash output with the published checksum
4. If hashes match exactly, the file is authentic and unmodified

**Example verification:**

```powershell
# Check a specific file
$hash = Get-FileHash .\HiveCode-0.1.0-x64-setup.exe -Algorithm SHA256
Write-Host "Downloaded file hash: $($hash.Hash)"

# If this matches the published checksum, the file is verified
```

### Where to Get Published Checksums

Published checksums for official HiveCode releases are available at:

- GitHub Releases: [https://github.com/HivePowered/HiveCode/releases](https://github.com/HivePowered/HiveCode/releases)
- HivePowered website: [https://hivepowered.ai](https://hivepowered.ai)

## Building and Verifying Locally

### Step 1: Build the Installers

```powershell
cd installers\windows
.\build.ps1
```

### Step 2: Generate Checksums

```powershell
cd output
Get-FileHash .\*.exe -Algorithm SHA256 | Format-Table -AutoSize
```

### Step 3: Store Checksums

Save the output to a secure location for distribution to users:

```powershell
Get-FileHash .\*.exe -Algorithm SHA256 | ForEach-Object {
    "$($_.Hash)  $(Split-Path -Leaf $_.Path)"
} | Out-File ..\CHECKSUMS_v0.1.0.txt -Encoding UTF8
```

## Integrity Verification Best Practices

1. **Always verify checksums for downloaded files** before running them
2. **Use SHA256 at minimum** (all HiveCode installers use SHA256)
3. **Compare full hashes** - a single character difference indicates tampering or corruption
4. **Keep checksums in a separate location** from the installers themselves
5. **For releases, have checksums signed** with a GPG key (future versions)

## Hash Algorithm Details

- **Algorithm**: SHA256 (part of SHA-2 family)
- **Output**: 64-character hexadecimal string
- **Collision resistance**: Cryptographically secure for malware detection
- **Not a signature**: Checksums prove integrity but not authenticity (GPG signing recommended for releases)

## Troubleshooting

### Checksum Mismatch

If your generated checksum doesn't match the published version:

1. **Delete and re-download** the installer (possible corruption)
2. **Rebuild locally** to generate new checksums
3. **Check disk integrity** with `chkdsk C: /F` if corruption is suspected
4. **Report the issue** at [https://github.com/HivePowered/HiveCode/issues](https://github.com/HivePowered/HiveCode/issues)

### PowerShell Command Not Working

If `Get-FileHash` returns an error:

- Ensure you're using **PowerShell 4.0+** (run `$PSVersionTable.PSVersion`)
- On older Windows, use this alternative:
  ```powershell
  certUtil -hashfile .\HiveCode-0.1.0-x64-setup.exe SHA256
  ```

## Related Files

- `build.ps1` - Build automation script
- `installer.iss` - Inno Setup configuration
- `README.md` - Build instructions and overview
- `../../LICENSE` - HiveCode license file
