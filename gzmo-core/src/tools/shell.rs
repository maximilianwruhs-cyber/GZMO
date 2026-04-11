//! # Shell Execution Tool
//!
//! Execute shell commands in a sandboxed environment.
//! Host-mode for now, Docker/gVisor isolation in Phase 3.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use std::time::Duration;

use crate::tools::{ToolDef, ToolHandler};

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

        // ─── SECURITY BLACKLIST FOR DAEMON SAFETY ───
        let blacklist = ["rm -rf", "dd ", "mkfs", "shutdown", "reboot", "init 0"];
        let cmd_lower = command.to_lowercase();
        for blocked in &blacklist {
            if cmd_lower.contains(blocked) {
                tracing::warn!(command = %command, "Blocked execution: matched destructive blacklist");
                return Ok(format!("ERROR: Command '{}' is BLACKLISTED for safety reasons and cannot be executed.", command));
            }
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
