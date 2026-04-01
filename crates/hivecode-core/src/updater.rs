//! Auto-updater system for HiveCode
//!
//! Provides automatic checking and installation of updates with:
//! - Multiple update channels (stable, beta, nightly)
//! - Semantic version comparison
//! - GitHub releases API integration
//! - Update download and installation
//! - Background update checking

use crate::error::{HiveCodeError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tracing::{debug, info, warn};

/// Information about an available update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// Current version that's running
    pub current_version: String,

    /// Latest available version
    pub latest_version: String,

    /// Release notes or changelog
    pub release_notes: String,

    /// URL to download the update
    pub download_url: String,

    /// ISO 8601 timestamp of when release was published
    pub published_at: String,

    /// Whether this is a mandatory update
    pub is_mandatory: bool,

    /// Checksum (SHA256) of the download for verification
    pub checksum: Option<String>,

    /// File size in bytes
    pub file_size: Option<u64>,
}

/// Update channel for version selection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum UpdateChannel {
    /// Stable releases only
    Stable,

    /// Beta/preview releases
    Beta,

    /// Nightly/development releases
    Nightly,
}

impl std::fmt::Display for UpdateChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stable => write!(f, "stable"),
            Self::Beta => write!(f, "beta"),
            Self::Nightly => write!(f, "nightly"),
        }
    }
}

impl std::str::FromStr for UpdateChannel {
    type Err = HiveCodeError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "stable" => Ok(Self::Stable),
            "beta" => Ok(Self::Beta),
            "nightly" => Ok(Self::Nightly),
            _ => Err(HiveCodeError::ConfigError(format!("Unknown channel: {}", s))),
        }
    }
}

/// GitHub releases API response
#[derive(Debug, Clone, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: String,
    published_at: String,
    assets: Vec<GitHubAsset>,
    prerelease: bool,
    draft: bool,
}

/// GitHub release asset
#[derive(Debug, Clone, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

/// Auto-update manager
pub struct UpdateManager {
    /// Current running version
    current_version: String,

    /// Preferred update channel
    channel: UpdateChannel,

    /// GitHub releases API endpoint
    github_repo: String, // inspireyourbrand-dev/hivecode

    /// HTTP client for API requests
    http_client: reqwest::Client,

    /// Whether auto-check is enabled
    auto_check_enabled: bool,

    /// How often to check (in hours)
    check_interval_hours: u32,
}

impl UpdateManager {
    /// Create a new update manager
    pub fn new(current_version: impl Into<String>) -> Self {
        Self {
            current_version: current_version.into(),
            channel: UpdateChannel::Stable,
            github_repo: "inspireyourbrand-dev/hivecode".to_string(),
            http_client: reqwest::Client::new(),
            auto_check_enabled: true,
            check_interval_hours: 24,
        }
    }

    /// Check for available updates asynchronously
    pub async fn check_for_updates(&self) -> Result<Option<UpdateInfo>> {
        info!("Checking for updates on {} channel", self.channel);

        let releases = self.fetch_github_releases().await?;

        // Filter releases based on channel and prerelease status
        let filtered = releases
            .into_iter()
            .filter(|r| {
                !r.draft && match self.channel {
                    UpdateChannel::Stable => !r.prerelease,
                    UpdateChannel::Beta => true, // Include everything for beta
                    UpdateChannel::Nightly => true,
                }
            })
            .collect::<Vec<_>>();

        if filtered.is_empty() {
            debug!("No releases found on {} channel", self.channel);
            return Ok(None);
        }

        // Find the latest compatible release
        for release in filtered {
            // Clean up tag name (remove 'v' prefix)
            let version = release.tag_name.trim_start_matches('v');

            if Self::is_newer_version(&self.current_version, version) {
                // Find the binary asset
                let asset = release
                    .assets
                    .iter()
                    .find(|a| self.is_compatible_asset(&a.name))
                    .ok_or_else(|| {
                        HiveCodeError::UpdateError(
                            format!("No compatible binary found for version {}", version),
                        )
                    })?;

                info!("Found update: {} -> {}", self.current_version, version);

                return Ok(Some(UpdateInfo {
                    current_version: self.current_version.clone(),
                    latest_version: version.to_string(),
                    release_notes: release.body,
                    download_url: asset.browser_download_url.clone(),
                    published_at: release.published_at,
                    is_mandatory: Self::is_mandatory_update(&self.current_version, version),
                    checksum: None,
                    file_size: Some(asset.size),
                }));
            }
        }

        debug!("No newer version available");
        Ok(None)
    }

    /// Download an update to a temporary location
    pub async fn download_update(&self, info: &UpdateInfo) -> Result<PathBuf> {
        info!("Downloading update from {}", info.download_url);

        let response = self.http_client
            .get(&info.download_url)
            .send()
            .await
            .map_err(|e| HiveCodeError::UpdateError(format!("Download failed: {}", e)))?;

        // Create temp directory for download
        let temp_dir = std::env::temp_dir().join("hivecode-updates");
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| HiveCodeError::UpdateError(format!("Failed to create temp dir: {}", e)))?;

        let file_name = format!("hivecode-{}", info.latest_version);
        let file_path = temp_dir.join(&file_name);

        // Download file
        let bytes = response.bytes().await
            .map_err(|e| HiveCodeError::UpdateError(format!("Failed to read response: {}", e)))?;

        tokio::fs::write(&file_path, bytes)
            .await
            .map_err(|e| HiveCodeError::UpdateError(format!("Failed to write file: {}", e)))?;

        info!("Update downloaded to {:?}", file_path);
        Ok(file_path)
    }

    /// Apply the downloaded update
    pub async fn apply_update(&self, binary_path: &PathBuf) -> Result<()> {
        info!("Applying update from {:?}", binary_path);

        if !binary_path.exists() {
            return Err(HiveCodeError::UpdateError(
                "Update binary not found".to_string(),
            ));
        }

        // Make binary executable (on Unix)
        #[cfg(unix)]
        {
            use std::fs::Permissions;
            use std::os::unix::fs::PermissionsExt;

            let perms = Permissions::from_mode(0o755);
            std::fs::set_permissions(binary_path, perms)
                .map_err(|e| HiveCodeError::UpdateError(format!("Failed to make executable: {}", e)))?;
        }

        // Get current executable path
        let current_exe = std::env::current_exe()
            .map_err(|e| HiveCodeError::UpdateError(format!("Cannot determine current executable: {}", e)))?;

        // Create backup
        let backup_path = current_exe.with_extension("backup");
        std::fs::copy(&current_exe, &backup_path)
            .map_err(|e| HiveCodeError::UpdateError(format!("Failed to create backup: {}", e)))?;

        // Replace binary
        std::fs::rename(binary_path, &current_exe)
            .map_err(|e| {
                // Try to restore backup if replacement fails
                let _ = std::fs::rename(&backup_path, &current_exe);
                HiveCodeError::UpdateError(format!("Failed to install update: {}", e))
            })?;

        info!("Update applied successfully. Restart required.");

        // Note: In a real application, we'd restart the app here
        // For now, just indicate success

        Ok(())
    }

    /// Set the update channel
    pub fn set_channel(&mut self, channel: UpdateChannel) {
        info!("Update channel changed to {}", channel);
        self.channel = channel;
    }

    /// Get the current update channel
    pub fn get_channel(&self) -> UpdateChannel {
        self.channel
    }

    /// Check if auto-update checking is enabled
    pub fn auto_check_enabled(&self) -> bool {
        self.auto_check_enabled
    }

    /// Enable or disable auto-update checking
    pub fn set_auto_check(&mut self, enabled: bool) {
        self.auto_check_enabled = enabled;
        info!("Auto-update checking: {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Set the auto-check interval
    pub fn set_check_interval(&mut self, hours: u32) {
        self.check_interval_hours = hours;
        debug!("Check interval set to {} hours", hours);
    }

    /// Fetch releases from GitHub API
    async fn fetch_github_releases(&self) -> Result<Vec<GitHubRelease>> {
        let url = format!(
            "https://api.github.com/repos/{}/releases",
            self.github_repo
        );

        let response = self.http_client
            .get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| HiveCodeError::UpdateError(format!("GitHub API request failed: {}", e)))?;

        let releases: Vec<GitHubRelease> = response.json().await
            .map_err(|e| HiveCodeError::UpdateError(format!("Failed to parse releases: {}", e)))?;

        Ok(releases)
    }

    /// Compare semantic versions
    /// Returns true if new_version > current_version
    fn is_newer_version(current: &str, new: &str) -> bool {
        // Simple semver comparison: split by . and compare
        let current_parts: Vec<&str> = current.split('.').collect();
        let new_parts: Vec<&str> = new.split('.').collect();

        for (i, new_part) in new_parts.iter().enumerate() {
            let current_part = current_parts.get(i).copied().unwrap_or("0");

            // Extract numeric portion and compare
            if let (Ok(curr_num), Ok(new_num)) = (
                current_part.split('-').next().unwrap_or("0").parse::<u32>(),
                new_part.split('-').next().unwrap_or("0").parse::<u32>(),
            ) {
                if new_num > curr_num {
                    return true;
                } else if new_num < curr_num {
                    return false;
                }
            }
        }

        false
    }

    /// Check if an asset name is compatible with the current system
    fn is_compatible_asset(&self, asset_name: &str) -> bool {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        // Look for platform-specific binaries
        match (os, arch) {
            ("macos", "x86_64") => asset_name.contains("darwin-x64") || asset_name.contains("macos-x64"),
            ("macos", "aarch64") => asset_name.contains("darwin-arm64") || asset_name.contains("macos-arm64"),
            ("linux", "x86_64") => asset_name.contains("linux-x64") || asset_name.contains("linux-gnu"),
            ("linux", "aarch64") => asset_name.contains("linux-arm64"),
            ("windows", "x86_64") => asset_name.contains("win32-x64") || asset_name.contains("windows-x64"),
            _ => asset_name.contains("universal") || asset_name.contains("portable"),
        }
    }

    /// Determine if an update is mandatory
    fn is_mandatory_update(current: &str, new: &str) -> bool {
        // Mark as mandatory if it's a major version bump
        let current_parts: Vec<&str> = current.split('.').collect();
        let new_parts: Vec<&str> = new.split('.').collect();

        if let (Some(curr_major), Some(new_major)) = (
            current_parts.first().and_then(|p| p.parse::<u32>().ok()),
            new_parts.first().and_then(|p| p.parse::<u32>().ok()),
        ) {
            new_major > curr_major
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_channel_display() {
        assert_eq!(UpdateChannel::Stable.to_string(), "stable");
        assert_eq!(UpdateChannel::Beta.to_string(), "beta");
        assert_eq!(UpdateChannel::Nightly.to_string(), "nightly");
    }

    #[test]
    fn test_update_channel_parsing() {
        assert_eq!(UpdateChannel::from_str("stable").unwrap(), UpdateChannel::Stable);
        assert_eq!(UpdateChannel::from_str("BETA").unwrap(), UpdateChannel::Beta);
        assert_eq!(UpdateChannel::from_str("nightly").unwrap(), UpdateChannel::Nightly);
        assert!(UpdateChannel::from_str("invalid").is_err());
    }

    #[test]
    fn test_version_comparison() {
        assert!(UpdateManager::is_newer_version("0.1.0", "0.2.0"));
        assert!(UpdateManager::is_newer_version("1.0.0", "2.0.0"));
        assert!(UpdateManager::is_newer_version("0.1.5", "0.2.0"));
        assert!(!UpdateManager::is_newer_version("0.2.0", "0.1.0"));
        assert!(!UpdateManager::is_newer_version("0.2.0", "0.2.0"));
    }

    #[test]
    fn test_asset_compatibility() {
        let manager = UpdateManager::new("0.1.0");

        #[cfg(target_os = "macos")]
        {
            #[cfg(target_arch = "x86_64")]
            assert!(manager.is_compatible_asset("hivecode-darwin-x64"));
            #[cfg(target_arch = "aarch64")]
            assert!(manager.is_compatible_asset("hivecode-darwin-arm64"));
        }

        #[cfg(target_os = "linux")]
        {
            #[cfg(target_arch = "x86_64")]
            assert!(manager.is_compatible_asset("hivecode-linux-x64"));
        }
    }

    #[test]
    fn test_mandatory_update() {
        assert!(UpdateManager::is_mandatory_update("1.0.0", "2.0.0"));
        assert!(!UpdateManager::is_mandatory_update("1.0.0", "1.1.0"));
        assert!(!UpdateManager::is_mandatory_update("2.0.0", "2.1.0"));
    }

    #[test]
    fn test_update_manager_creation() {
        let manager = UpdateManager::new("0.1.0");
        assert_eq!(manager.current_version, "0.1.0");
        assert_eq!(manager.get_channel(), UpdateChannel::Stable);
        assert!(manager.auto_check_enabled());
    }
}
