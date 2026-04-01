/// Risk classification for shell commands
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CommandRisk {
    Safe,
    Moderate,
    Dangerous,
    Blocked,
}

/// Checker for shell command safety
pub struct ShellSecurityChecker {
    safe_patterns: Vec<&'static str>,
    moderate_patterns: Vec<&'static str>,
    dangerous_patterns: Vec<&'static str>,
    blocked_patterns: Vec<&'static str>,
}

impl ShellSecurityChecker {
    pub fn new() -> Self {
        Self {
            safe_patterns: vec![
                "echo",
                "cat",
                "ls",
                "pwd",
                "date",
                "whoami",
                "id",
                "ps",
                "grep",
                "cut",
                "head",
                "tail",
                "sort",
                "uniq",
                "wc",
                "find",
                "stat",
                "file",
                "which",
                "git status",
                "git log",
                "git diff",
                "git branch",
                "git show",
                "git blame",
                "npm list",
                "npm info",
                "npm search",
                "cargo check",
                "cargo test",
                "cargo build",
                "cargo doc",
            ],
            moderate_patterns: vec![
                "curl",
                "wget",
                "scp",
                "rsync",
                "tar",
                "zip",
                "gzip",
                "gunzip",
                "mv",
                "cp",
                "mkdir",
                "touch",
                "chmod",
                "chown",
                "useradd",
                "git clone",
                "git pull",
                "git push",
                "npm install",
                "npm update",
                "cargo install",
                "make",
                "gcc",
                "clang",
            ],
            dangerous_patterns: vec![
                "rm ",
                "rm -r",
                "rm -f",
                "rmdir",
                "dd ",
                "mkfs",
                "fdisk",
                "parted",
                "mount",
                "umount",
                "kill ",
                "killall",
                "service",
                "systemctl",
                "reboot",
                "shutdown",
                "sudo",
                "su ",
                "passwd",
                "usermod",
                "userdel",
                "groupdel",
                "vi ",
                "vim ",
                "sed ",
                "awk ",
                "perl ",
                "python -c",
                "ruby -e",
                "node -e",
                "eval",
                "exec",
                "source ",
                ". ",
                "bash -c",
                "sh -c",
                "bash <",
                "sh <",
            ],
            blocked_patterns: vec![
                "curl | bash",
                "wget | bash",
                "curl | sh",
                "wget | sh",
                "> /dev/sda",
                "> /dev/hda",
                "dd if=/dev/zero of=/",
                "rm -rf /",
                "fork() {",
                ":(){ :|:& };:",
                ":(){:|:&};:",
            ],
        }
    }

    /// Classify the risk level of a command
    pub fn classify_command(&self, command: &str) -> CommandRisk {
        let lower_command = command.to_lowercase();

        // Check blocked patterns first
        for pattern in &self.blocked_patterns {
            if lower_command.contains(pattern) {
                return CommandRisk::Blocked;
            }
        }

        // Check dangerous patterns
        for pattern in &self.dangerous_patterns {
            if lower_command.contains(pattern) {
                // Special handling: rm alone is dangerous, but some patterns like
                // "remove" or "remove_temp" should be allowed
                if pattern == &"rm " {
                    if lower_command.contains("rm ") {
                        return CommandRisk::Dangerous;
                    }
                } else {
                    return CommandRisk::Dangerous;
                }
            }
        }

        // Check moderate patterns
        for pattern in &self.moderate_patterns {
            if lower_command.contains(pattern) {
                return CommandRisk::Moderate;
            }
        }

        // Check safe patterns
        for pattern in &self.safe_patterns {
            if lower_command.contains(pattern) {
                return CommandRisk::Safe;
            }
        }

        // Default to moderate for unknown commands
        CommandRisk::Moderate
    }
}

impl Default for ShellSecurityChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_safe_commands() {
        let checker = ShellSecurityChecker::new();

        assert_eq!(checker.classify_command("echo hello"), CommandRisk::Safe);
        assert_eq!(checker.classify_command("cat /etc/hosts"), CommandRisk::Safe);
        assert_eq!(checker.classify_command("ls -la"), CommandRisk::Safe);
        assert_eq!(checker.classify_command("git log"), CommandRisk::Safe);
        assert_eq!(checker.classify_command("git status"), CommandRisk::Safe);
    }

    #[test]
    fn test_classify_moderate_commands() {
        let checker = ShellSecurityChecker::new();

        assert_eq!(checker.classify_command("curl https://example.com"), CommandRisk::Moderate);
        assert_eq!(checker.classify_command("mkdir /tmp/test"), CommandRisk::Moderate);
        assert_eq!(checker.classify_command("cp file.txt copy.txt"), CommandRisk::Moderate);
        assert_eq!(checker.classify_command("npm install"), CommandRisk::Moderate);
    }

    #[test]
    fn test_classify_dangerous_commands() {
        let checker = ShellSecurityChecker::new();

        assert_eq!(checker.classify_command("rm -rf /tmp"), CommandRisk::Dangerous);
        assert_eq!(checker.classify_command("sudo apt-get update"), CommandRisk::Dangerous);
        assert_eq!(checker.classify_command("kill -9 1234"), CommandRisk::Dangerous);
        assert_eq!(checker.classify_command("sed -i 's/a/b/' file"), CommandRisk::Dangerous);
    }

    #[test]
    fn test_classify_blocked_commands() {
        let checker = ShellSecurityChecker::new();

        assert_eq!(checker.classify_command("curl | bash"), CommandRisk::Blocked);
        assert_eq!(checker.classify_command("wget | sh"), CommandRisk::Blocked);
        assert_eq!(checker.classify_command("rm -rf /"), CommandRisk::Blocked);
        assert_eq!(
            checker.classify_command("dd if=/dev/zero of=/dev/sda"),
            CommandRisk::Blocked
        );
    }

    #[test]
    fn test_classify_case_insensitive() {
        let checker = ShellSecurityChecker::new();

        assert_eq!(checker.classify_command("Echo Hello"), CommandRisk::Safe);
        assert_eq!(checker.classify_command("CURL https://example.com"), CommandRisk::Moderate);
        assert_eq!(checker.classify_command("RM -RF /"), CommandRisk::Blocked);
    }

    #[test]
    fn test_classify_unknown_command() {
        let checker = ShellSecurityChecker::new();

        // Unknown commands default to moderate
        assert_eq!(
            checker.classify_command("unknowncommand arg1 arg2"),
            CommandRisk::Moderate
        );
    }
}
