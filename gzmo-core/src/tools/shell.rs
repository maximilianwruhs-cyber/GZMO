//! # Shell Execution Tool
//!
//! Execute shell commands in a sandboxed environment.
//! Uses an **allowlist** model: only commands whose first token matches a
//! known-safe prefix are permitted. This prevents the LLM from hallucinating
//! destructive commands (rm, dd, mkfs, etc.) in daemon mode.
//!
//! Host-mode for now, Docker/gVisor isolation in Phase 3.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use std::time::Duration;

use crate::tools::{ToolDef, ToolHandler};

/// Safe command prefixes. Only the first whitespace-delimited token of the
/// command is checked against this list. Anything outside is rejected.
const SAFE_COMMAND_PREFIXES: &[&str] = &[
    // Filesystem inspection (read-only)
    "ls", "cat", "head", "tail", "wc", "stat", "file", "du", "df",
    "find", "locate", "tree", "readlink", "realpath", "basename", "dirname",
    // Text processing (read-only)
    "grep", "rg", "awk", "sed", "sort", "uniq", "cut", "tr", "jq",
    "diff", "comm", "paste", "column", "fold", "fmt", "tee",
    // System inspection
    "ps", "top", "htop", "uname", "hostname", "whoami", "id", "uptime",
    "date", "cal", "env", "printenv", "lsblk", "lscpu", "lsusb", "lspci",
    "free", "vmstat", "iostat", "ip", "ss", "netstat",
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
    // GZMO-specific
    "gzmo",
];

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
            description: "Execute a shell command and return stdout/stderr. Timeout: 30s. Use for system inspection, file operations, and tool invocation.".to_string(),
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
        // Extract the first token (the actual binary being invoked).
        // Handles leading env vars like `FOO=bar cmd` and path prefixes like `/usr/bin/ls`.
        let first_token = command
            .split_whitespace()
            .find(|t| !t.contains('='))  // skip env var assignments
            .unwrap_or("");
        // Strip any path prefix: "/usr/bin/ls" → "ls"
        let binary_name = first_token.rsplit('/').next().unwrap_or(first_token);

        if !SAFE_COMMAND_PREFIXES.iter().any(|safe| binary_name == *safe) {
            tracing::warn!(command = %command, binary = %binary_name, "Blocked: not in allowlist");
            return Ok(format!(
                "ERROR: Command '{}' is not in the safe command allowlist. \
                Permitted commands: {}. \
                If you need to run this command, ask the user to execute it manually.",
                command,
                SAFE_COMMAND_PREFIXES.join(", ")
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
