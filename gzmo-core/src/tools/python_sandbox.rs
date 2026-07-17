//! # Python Sandbox Execution Tool
//!
//! Execute restricted Python 3 scripts in a secured sub-process.
//! Implements strict input scanning to block forbidden imports (e.g. os, subprocess, sys),
//! and imposes limits on code length, execution timeout, and output size.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::json;
use std::time::Duration;

use super::{ToolDef, ToolHandler};
use crate::config::PedagogyConfig;

/// Securely execute Python code.
pub struct PythonSandboxTool {
    pub enabled: bool,
    pub max_code_chars: usize,
    pub timeout: Duration,
    pub max_output_chars: usize,
    pub blocked_imports: Vec<String>,
}

impl Default for PythonSandboxTool {
    fn default() -> Self {
        Self {
            enabled: true,
            max_code_chars: 2000,
            timeout: Duration::from_secs(10),
            max_output_chars: 4000,
            blocked_imports: vec![
                "os".to_string(),
                "subprocess".to_string(),
                "socket".to_string(),
                "shutil".to_string(),
                "sys".to_string(),
                "pathlib".to_string(),
                "importlib".to_string(),
                "__import__".to_string(),
                "open".to_string(),
            ],
        }
    }
}

impl PythonSandboxTool {
    pub fn new(config: &PedagogyConfig) -> Self {
        let s = &config.sandbox;
        Self {
            enabled: s.enabled,
            max_code_chars: s.max_code_chars,
            timeout: Duration::from_secs(s.timeout_secs),
            max_output_chars: s.max_output_chars,
            blocked_imports: s.blocked_imports.clone(),
        }
    }

    /// Scan code for blocked words at token boundaries.
    pub fn is_blocked(&self, code: &str) -> Option<String> {
        for word in &self.blocked_imports {
            if has_blocked_word(code, word) {
                return Some(word.clone());
            }
        }
        None
    }
}

/// Helper function to search for a word strictly bounded by token boundaries in Python.
fn has_blocked_word(code: &str, word: &str) -> bool {
    let mut start = 0;
    while let Some(pos) = code[start..].find(word) {
        let actual_pos = start + pos;
        let char_before = if actual_pos > 0 {
            code.as_bytes().get(actual_pos - 1).map(|&b| b as char)
        } else {
            None
        };
        let char_after = code
            .as_bytes()
            .get(actual_pos + word.len())
            .map(|&b| b as char);

        let is_boundary_before = match char_before {
            Some(c) => !c.is_alphanumeric() && c != '_',
            None => true,
        };
        let is_boundary_after = match char_after {
            Some(c) => !c.is_alphanumeric() && c != '_',
            None => true,
        };

        if is_boundary_before && is_boundary_after {
            return true;
        }
        start = actual_pos + 1;
    }
    false
}

#[async_trait]
impl ToolHandler for PythonSandboxTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "python_sandbox".to_string(),
            description: "Execute a short Python 3 script in a secure sandbox and return stdout + stderr. \
                          Allowed libraries: math, statistics, fractions, decimal, json, re, itertools. \
                          Filesystem writes, subprocesses, and network access are blocked.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "The Python 3 code to execute (maximum 2000 characters)"
                    }
                },
                "required": ["code"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        if !self.enabled {
            return Err(anyhow!("Python sandbox execution is disabled."));
        }

        let code = args["code"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'code' argument"))?;

        if code.len() > self.max_code_chars {
            return Err(anyhow!(
                "Code length of {} exceeds the maximum limit of {} characters.",
                code.len(),
                self.max_code_chars
            ));
        }

        if let Some(blocked) = self.is_blocked(code) {
            return Err(anyhow!(
                "Security Block: use of forbidden module/keyword '{}' is not allowed in sandbox.",
                blocked
            ));
        }

        let mut cmd = tokio::process::Command::new("python3");
        cmd.arg("-c").arg(code);

        let result = tokio::time::timeout(self.timeout, cmd.output()).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);

                let mut response = format!("Exit code: {}\n", exit_code);

                let merged = if !stdout.is_empty() && !stderr.is_empty() {
                    format!("--- stdout ---\n{}\n--- stderr ---\n{}", stdout, stderr)
                } else if !stdout.is_empty() {
                    stdout.to_string()
                } else {
                    stderr.to_string()
                };

                let output_truncated = if merged.len() > self.max_output_chars {
                    format!(
                        "{}\n... [truncated, {} total chars]",
                        &merged[..self.max_output_chars],
                        merged.len()
                    )
                } else {
                    merged
                };

                response.push_str(&output_truncated);
                Ok(response)
            }
            Ok(Err(e)) => Err(anyhow!("Python execution failed to start: {}", e)),
            Err(_) => Err(anyhow!(
                "Python execution timed out after {:?}",
                self.timeout
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SandboxConfig;

    fn test_config() -> PedagogyConfig {
        PedagogyConfig {
            sandbox: SandboxConfig {
                enabled: true,
                max_code_chars: 50,
                timeout_secs: 2,
                max_output_chars: 20,
                blocked_imports: vec!["os".into(), "sys".into(), "open".into()],
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_blocked_word_scans() {
        let tool = PythonSandboxTool::new(&test_config());

        // Basic block
        assert_eq!(tool.is_blocked("import os"), Some("os".to_string()));
        assert_eq!(tool.is_blocked("import sys"), Some("sys".to_string()));

        // Bounded boundaries
        assert_eq!(tool.is_blocked("my_os_variable = 1"), None);
        assert_eq!(tool.is_blocked("clos = 3"), None);

        // String quotes and comments (blocked regardless of context for safety)
        assert_eq!(tool.is_blocked("print('os')"), Some("os".to_string()));
        assert_eq!(tool.is_blocked("# os is blocked"), Some("os".to_string()));

        // Bounded punctuation
        assert_eq!(tool.is_blocked("os.path.join()"), Some("os".to_string()));
        assert_eq!(
            tool.is_blocked("print(open('file'))"),
            Some("open".to_string())
        );
    }

    #[tokio::test]
    async fn test_sandbox_tool_limits() {
        let tool = PythonSandboxTool::new(&test_config());

        // Code too long
        let long_code = "print(1)".repeat(10);
        let res = tool.execute(json!({ "code": long_code })).await;
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("exceeds the maximum limit"));

        // Security block
        let res = tool.execute(json!({ "code": "import os" })).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Security Block"));

        // Normal execution
        let res = tool.execute(json!({ "code": "print(2 + 2)" })).await;
        assert!(res.is_ok());
        let out = res.unwrap();
        assert!(out.contains("Exit code: 0"));
        assert!(out.contains("4"));

        // Output truncation
        let res = tool.execute(json!({ "code": "print('x'*100)" })).await;
        assert!(res.is_ok());
        let out = res.unwrap();
        assert!(out.contains("[truncated"));
    }
}
