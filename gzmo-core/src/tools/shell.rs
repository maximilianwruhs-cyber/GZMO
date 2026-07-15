//! # Shell Execution Tool
//!
//! Execute shell commands in a sandboxed environment.
//! Uses an **allowlist** model for known-safe binaries, plus direct `.sh`
//! script invocation. `bash` is allowlisted for script and command execution.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use std::time::Duration;

use crate::tools::{ToolDef, ToolHandler};

/// Safe command prefixes. Only the first whitespace-delimited token of the
/// command is checked against this list. Shell scripts and script runners are
/// handled separately by [`is_command_allowed`].
const SAFE_COMMAND_PREFIXES: &[&str] = &[
    // Filesystem inspection (read-only)
    "ls", "cat", "head", "tail", "wc", "stat", "file", "du", "df",
    "find", "locate", "tree", "readlink", "realpath", "basename", "dirname",
    "pwd",
    // Text processing (read-only)
    "grep", "rg", "awk", "sed", "sort", "uniq", "cut", "tr", "jq",
    "diff", "comm", "paste", "column", "fold", "fmt", "tee",
    // System inspection
    "ps", "top", "htop", "uname", "hostname", "whoami", "id", "uptime",
    "date", "cal", "env", "printenv", "lsblk", "lscpu", "lsusb", "lspci",
    "free", "vmstat", "iostat", "ip", "ss", "netstat",
    "pgrep", "pidof", "journalctl", "systemctl", "nvidia-smi", "lsmod",
    // Containers / sidecars (read-only ops like `docker ps` — first-token gate only)
    "docker",
    // Data stores (inspection)
    "redis-cli", "sqlite3",
    // Development tools
    "git", "cargo", "rustc", "python3", "python", "node", "npm", "npx",
    "pip", "pip3", "make", "cmake",
    // Archive / compression (read-only ops)
    "tar", "gzip", "gunzip", "zcat", "bzip2", "xz",
    // Network (read-only, needed for cloud-mode API/web access)
    "curl", "wget",
    // Misc safe
    "echo", "printf", "true", "false", "test", "[", "which", "type",
    "man", "help", "sha256sum", "md5sum", "b2sum", "xxd", "hexdump",
    // Shell / scripts
    "bash",
    // GZMO-specific
    "gzmo",
];

fn first_command_token(command: &str) -> &str {
    command
        .split_whitespace()
        .find(|t| !t.contains('='))
        .unwrap_or("")
}

fn command_binary_name(token: &str) -> &str {
    token.rsplit('/').next().unwrap_or(token)
}

fn is_shell_script_path(token: &str) -> bool {
    token.ends_with(".sh")
}

/// Allow `bash script.sh` / `sh -x script.sh` but block inline `bash -c "..."`.
fn is_allowed_script_runner(command: &str) -> bool {
    let mut tokens = command
        .split_whitespace()
        .filter(|t| !t.contains('='));

    let Some(first) = tokens.next() else {
        return false;
    };

    let runner = command_binary_name(first);
    if runner != "bash" && runner != "sh" {
        return false;
    }

    for token in tokens {
        if token == "-c" {
            return false;
        }
        if token.starts_with('-') {
            continue;
        }
        return is_shell_script_path(token);
    }

    false
}

fn is_command_allowed(command: &str) -> bool {
    let first_token = first_command_token(command);
    let binary = command_binary_name(first_token);

    if SAFE_COMMAND_PREFIXES.iter().any(|safe| binary == *safe) {
        return true;
    }
    if is_shell_script_path(first_token) {
        return true;
    }
    is_allowed_script_runner(command)
}

/// Execute a shell command on the host.
/// Captures stdout + stderr with a timeout to prevent runaway processes.
pub struct ShellExecTool {
    /// Maximum execution time before killing the process
    pub timeout: Duration,
    /// Working directory for commands
    pub cwd: Option<String>,
}

impl Default for ShellExecTool {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            cwd: None,
        }
    }
}

#[async_trait]
impl ToolHandler for ShellExecTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "shell_exec".to_string(),
            description: "Execute a shell command and return stdout/stderr. Timeout: 30s. \
                `bash` and `.sh` scripts are allowed. Prefer `/status` or \
                `gzmo status` for ecosystem overview instead of ad-hoc shell probes.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute (runs via /bin/sh -c)"
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let command = args["command"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' argument"))?;

        // ─── SECURITY ALLOWLIST ───
        let first_token = first_command_token(command);
        let binary_name = command_binary_name(first_token);

        if !is_command_allowed(command) {
            tracing::debug!(command = %command, binary = %binary_name, "Blocked: not in allowlist");
            let hint = match binary_name {
                "status" => " Use `/status` in chat or `gzmo status` on the CLI.",
                "bash" => " Use `bash path/to/script.sh` or `./path/to/script.sh`.",
                _ => " For a full stack snapshot, use `/status` instead of many shell probes.",
            };
            return Ok(format!(
                "ERROR: Command blocked (binary '{binary_name}' not in allowlist).{hint} \
                Ask the user to run it manually if truly needed."
            ));
        }

        tracing::info!(command = %command, "Executing shell command");

        let mut cmd = tokio::process::Command::new("/bin/sh");
        cmd.arg("-c").arg(command);

        if let Some(cwd) = &self.cwd {
            cmd.current_dir(cwd);
        }

        // Execute with timeout
        let result = tokio::time::timeout(self.timeout, cmd.output()).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);

                let mut response = String::new();
                response.push_str(&format!("Exit code: {}\n", exit_code));

                if !stdout.is_empty() {
                    let stdout_truncated = if stdout.len() > 6000 {
                        format!(
                            "{}\n... [truncated, {} total chars]",
                            &stdout[..6000],
                            stdout.len()
                        )
                    } else {
                        stdout.to_string()
                    };
                    response.push_str(&format!("--- stdout ---\n{}\n", stdout_truncated));
                }

                if !stderr.is_empty() {
                    let stderr_truncated = if stderr.len() > 2000 {
                        format!(
                            "{}\n... [truncated, {} total chars]",
                            &stderr[..2000],
                            stderr.len()
                        )
                    } else {
                        stderr.to_string()
                    };
                    response.push_str(&format!("--- stderr ---\n{}\n", stderr_truncated));
                }

                Ok(response)
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("Command execution failed: {}", e)),
            Err(_) => Err(anyhow::anyhow!(
                "Command timed out after {:?}: {}",
                self.timeout,
                command
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_safe_binaries() {
        assert!(is_command_allowed("ls -la"));
        assert!(is_command_allowed("FOO=bar grep pattern file"));
    }

    #[test]
    fn allows_direct_shell_scripts() {
        assert!(is_command_allowed("./skills/skill_card.sh creature"));
        assert!(is_command_allowed("skills/skill_card.sh"));
    }

    #[test]
    fn allows_bash_and_sh_script_runners() {
        assert!(is_command_allowed("bash skills/skill_card.sh creature"));
        assert!(is_command_allowed("bash -x skills/skill_card.sh"));
        assert!(is_command_allowed("sh scripts/live-smoke-all.sh"));
        assert!(is_command_allowed("GZMO_LLM_URL=http://127.0.0.1:8000/v1 bash skills/skill_card.sh"));
    }

    #[test]
    fn blocks_inline_shell_and_unsafe_binaries() {
        assert!(is_command_allowed("bash -c \"echo ok\""));
        assert!(!is_command_allowed("sh -c 'echo pwned'"));
        assert!(!is_command_allowed("rm -rf /"));
    }
}
