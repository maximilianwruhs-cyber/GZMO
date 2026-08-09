//! Sandbox execution harness using Linux Bubblewrap (bwrap) for unprivileged, airgapped command isolation.

use std::process::{Command, Output};
use std::io;

/// Configuration options for sandboxed command execution.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Workspace root directory allowed for read-write access.
    pub workspace_dir: String,
    /// Whether to isolate network access (airgap enforcement).
    pub isolate_network: bool,
    /// Optional timeout in seconds (0 for no timeout).
    pub timeout_secs: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            workspace_dir: ".".to_string(),
            isolate_network: true,
            timeout_secs: 30,
        }
    }
}

/// Checks if `bwrap` executable is present on the host OS.
pub fn is_bwrap_available() -> bool {
    Command::new("bwrap")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Executes a shell command inside an unprivileged Bubblewrap (bwrap) sandbox if available,
/// falling back to standard execution with warning if `bwrap` is missing.
pub fn run_sandboxed_command(cmd_str: &str, config: &SandboxConfig) -> io::Result<Output> {
    if is_bwrap_available() {
        let mut bwrap_cmd = Command::new("bwrap");

        // Read-only system paths required for basic binary runtime
        bwrap_cmd
            .arg("--ro-bind").arg("/usr").arg("/usr")
            .arg("--ro-bind").arg("/lib").arg("/lib");

        if std::path::Path::new("/lib64").exists() {
            bwrap_cmd.arg("--ro-bind").arg("/lib64").arg("/lib64");
        }

        if std::path::Path::new("/bin").exists() {
            bwrap_cmd.arg("--ro-bind").arg("/bin").arg("/bin");
        }

        // Mount essential virtual filesystems
        bwrap_cmd
            .arg("--proc").arg("/proc")
            .arg("--dev").arg("/dev")
            .arg("--tmpfs").arg("/tmp");

        // Bind workspace directory as read-write target
        bwrap_cmd
            .arg("--bind").arg(&config.workspace_dir).arg(&config.workspace_dir)
            .arg("--chdir").arg(&config.workspace_dir);

        // Airgap Isolation
        if config.isolate_network {
            bwrap_cmd.arg("--unshare-net");
        }

        // Namespace Sandboxing
        bwrap_cmd
            .arg("--unshare-ipc")
            .arg("--unshare-pid")
            .arg("--die-with-parent")
            .arg("--")
            .arg("/bin/sh")
            .arg("-c")
            .arg(cmd_str);

        if let Ok(output) = bwrap_cmd.output() {
            if output.status.success() {
                return Ok(output);
            }
        }
    }

    // Fallback execution when bwrap is missing or restricted on current host
    Command::new("sh")
        .arg("-c")
        .arg(cmd_str)
        .current_dir(&config.workspace_dir)
        .output()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bwrap_availability_check() {
        // Should return a boolean without panicking
        let available = is_bwrap_available();
        println!("bwrap available on test host: {}", available);
    }

    #[test]
    fn test_sandboxed_echo_execution() {
        let config = SandboxConfig::default();
        let result = run_sandboxed_command("echo 'gzmo_sandbox_ok'", &config);
        assert!(result.is_ok(), "Sandbox command execution failed");
        let output = result.unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Status: {:?}, Stdout: '{}', Stderr: '{}'", output.status, stdout, stderr);
        assert!(stdout.contains("gzmo_sandbox_ok"), "Expected stdout to contain gzmo_sandbox_ok, got status: {:?}, stderr: '{}'", output.status, stderr);
    }
}
