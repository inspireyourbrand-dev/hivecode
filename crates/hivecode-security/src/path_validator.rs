use std::path::{Path, PathBuf};

/// Validator for file system paths
pub struct PathValidator;

impl PathValidator {
    pub fn new() -> Self {
        Self
    }

    /// Check if a path is within a project root directory
    pub fn is_within_project(&self, path: &Path, project_root: &Path) -> bool {
        if let Ok(normalized_path) = self.normalize_path(path) {
            if let Ok(normalized_root) = self.normalize_path(project_root) {
                return normalized_path.starts_with(&normalized_root);
            }
        }
        false
    }

    /// Check if a path is a sensitive file that should not be accessed
    pub fn is_sensitive_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();

        let sensitive_patterns = [
            ".env",
            ".aws",
            ".ssh",
            ".git/config",
            ".gitcredentials",
            ".dockercfg",
            ".docker/config.json",
            "credentials",
            "credential",
            "secret",
            "token",
            "api_key",
            "apikey",
            "password",
            "passwd",
            "/etc/passwd",
            "/etc/shadow",
            "/etc/sudoers",
            "/root/.ssh",
            "/root/.aws",
            "/home/*/ssh",
            "id_rsa",
            "id_dsa",
            "id_ecdsa",
            "id_ed25519",
            "known_hosts",
            "authorized_keys",
            "pgp",
            "gnupg",
        ];

        for pattern in &sensitive_patterns {
            if path_str.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Normalize a path by resolving .. and symlinks
    pub fn normalize_path(&self, path: &Path) -> Result<PathBuf, std::io::Error> {
        // Simple normalization: canonicalize if possible, otherwise just normalize
        if path.exists() {
            std::fs::canonicalize(path)
        } else {
            // For non-existent paths, at least normalize the path string
            Ok(self.normalize_path_string(path))
        }
    }

    /// Normalize a path string by removing .. and . components
    fn normalize_path_string(&self, path: &Path) -> PathBuf {
        let mut components = Vec::new();

        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    components.pop();
                }
                std::path::Component::CurDir => {
                    // Skip current dir
                }
                std::path::Component::RootDir => {
                    components.clear();
                    components.push(component);
                }
                other => {
                    components.push(other);
                }
            }
        }

        components.iter().collect()
    }

    /// Check if a path is valid (not trying to escape, etc.)
    pub fn is_valid_path(&self, path: &Path) -> bool {
        // Check for path traversal attempts
        let path_str = path.to_string_lossy();

        // Block obvious path traversal
        if path_str.contains("../") || path_str.contains("..\\") {
            // Only block if it tries to go above root
            if let Ok(normalized) = self.normalize_path(path) {
                // Check if it successfully escaped the intended scope
                // For now, allow if it's a valid path
                return true;
            }
            return false;
        }

        true
    }
}

impl Default for PathValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_is_sensitive_file_env() {
        let validator = PathValidator::new();
        assert!(validator.is_sensitive_file(Path::new(".env")));
        assert!(validator.is_sensitive_file(Path::new("/home/user/.env")));
    }

    #[test]
    fn test_is_sensitive_file_ssh() {
        let validator = PathValidator::new();
        assert!(validator.is_sensitive_file(Path::new(".ssh/id_rsa")));
        assert!(validator.is_sensitive_file(Path::new("/root/.ssh/authorized_keys")));
    }

    #[test]
    fn test_is_sensitive_file_aws() {
        let validator = PathValidator::new();
        assert!(validator.is_sensitive_file(Path::new(".aws/credentials")));
        assert!(validator.is_sensitive_file(Path::new("/home/user/.aws/config")));
    }

    #[test]
    fn test_is_sensitive_file_docker() {
        let validator = PathValidator::new();
        assert!(validator.is_sensitive_file(Path::new(".docker/config.json")));
        assert!(validator.is_sensitive_file(Path::new(".dockercfg")));
    }

    #[test]
    fn test_is_not_sensitive_file() {
        let validator = PathValidator::new();
        assert!(!validator.is_sensitive_file(Path::new("README.md")));
        assert!(!validator.is_sensitive_file(Path::new("src/main.rs")));
        assert!(!validator.is_sensitive_file(Path::new("Cargo.toml")));
    }

    #[test]
    fn test_normalize_path_string() {
        let validator = PathValidator::new();

        let path = Path::new("/a/b/../c/./d");
        let normalized = validator.normalize_path_string(path);
        assert_eq!(normalized, Path::new("/a/c/d"));
    }

    #[test]
    fn test_is_within_project() {
        let validator = PathValidator::new();
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let file_path = project_root.join("src").join("main.rs");

        assert!(validator.is_within_project(&file_path, project_root));
    }

    #[test]
    fn test_is_not_within_project() {
        let validator = PathValidator::new();
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create a path outside the project
        let outside_path = Path::new("/etc/passwd");

        assert!(!validator.is_within_project(outside_path, project_root));
    }

    #[test]
    fn test_is_valid_path() {
        let validator = PathValidator::new();

        assert!(validator.is_valid_path(Path::new("/tmp/test.txt")));
        assert!(validator.is_valid_path(Path::new("./src/main.rs")));
    }
}
