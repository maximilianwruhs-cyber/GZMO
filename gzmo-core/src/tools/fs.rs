//! # Filesystem Tool
//!
//! Read, write, list, and search files on the local filesystem.
//! The agent's primary way to inspect and manipulate its environment.

use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;

use crate::tools::jail::PathJail;
use crate::tools::{ToolDef, ToolHandler};

fn check_path(jail: &Option<Arc<PathJail>>, path: &str) -> Result<String> {
    match jail {
        Some(j) => Ok(j.check(path)?.to_string_lossy().into_owned()),
        None => Ok(path.to_string()),
    }
}

// ─── Read File ──────────────────────────────────────────────────────────

pub struct FileReadTool {
    pub jail: Option<Arc<PathJail>>,
}

impl Default for FileReadTool {
    fn default() -> Self {
        Self { jail: None }
    }
}

#[async_trait]
impl ToolHandler for FileReadTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "file_read".to_string(),
            description: "Read the contents of a file at the given path. Returns the file content as text.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Absolute or relative path to the file to read"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' argument"))?;
        let path = check_path(&self.jail, path)?;

        let content = tokio::fs::read_to_string(&path).await?;
        Ok(if content.len() > 8000 {
            format!(
                "{}\n\n... [truncated at 8000 chars, total {} chars]",
                &content[..8000],
                content.len()
            )
        } else {
            content
        })
    }
}

// ─── Write File ─────────────────────────────────────────────────────────

pub struct FileWriteTool {
    pub jail: Option<Arc<PathJail>>,
}

impl Default for FileWriteTool {
    fn default() -> Self {
        Self { jail: None }
    }
}

#[async_trait]
impl ToolHandler for FileWriteTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "file_write".to_string(),
            description: "Write content to a file. Creates the file if it doesn't exist, overwrites if it does. Creates parent directories as needed.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Absolute or relative path to write to"
                    },
                    "content": {
                        "type": "string",
                        "description": "The content to write to the file"
                    }
                },
                "required": ["path", "content"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' argument"))?;
        let content = args["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' argument"))?;
        let path = check_path(&self.jail, path)?;

        if let Some(parent) = Path::new(&path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&path, content).await?;
        Ok(format!("Written {} bytes to {}", content.len(), path))
    }
}

// ─── List Directory ─────────────────────────────────────────────────────

pub struct DirListTool {
    pub jail: Option<Arc<PathJail>>,
}

impl Default for DirListTool {
    fn default() -> Self {
        Self { jail: None }
    }
}

#[async_trait]
impl ToolHandler for DirListTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "dir_list".to_string(),
            description: "List the contents of a directory. Returns filenames with types (file/dir) and sizes.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the directory to list"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' argument"))?;
        let path = check_path(&self.jail, path)?;

        let mut entries = tokio::fs::read_dir(&path).await?;
        let mut output = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let meta = entry.metadata().await?;
            let kind = if meta.is_dir() { "dir" } else { "file" };
            let size = if meta.is_file() {
                format!(" ({} bytes)", meta.len())
            } else {
                String::new()
            };
            output.push(format!(
                "  {} {kind}{size}",
                entry.file_name().to_string_lossy()
            ));
        }

        output.sort();
        if output.is_empty() {
            Ok("(empty directory)".to_string())
        } else {
            Ok(output.join("\n"))
        }
    }
}

// ─── File Search (grep) ─────────────────────────────────────────────────

pub struct FileSearchTool {
    pub jail: Option<Arc<PathJail>>,
}

impl Default for FileSearchTool {
    fn default() -> Self {
        Self { jail: None }
    }
}

#[async_trait]
impl ToolHandler for FileSearchTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "file_search".to_string(),
            description: "Search for a text pattern in files within a directory. Returns matching lines with file paths and line numbers.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory to search in"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Text pattern to search for (case-insensitive)"
                    }
                },
                "required": ["path", "pattern"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' argument"))?;
        let pattern = args["pattern"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'pattern' argument"))?;
        let path = check_path(&self.jail, path)?;

        let output = tokio::process::Command::new("grep")
            .args([
                "-rn",
                "-i",
                "--include=*.rs",
                "--include=*.md",
                "--include=*.toml",
                "--include=*.json",
                "--include=*.yaml",
                "--include=*.yml",
                "--include=*.txt",
                "--include=*.sh",
                pattern,
                &path,
            ])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.is_empty() {
            Ok(format!("No matches for '{}' in {}", pattern, path))
        } else if stdout.len() > 5000 {
            Ok(format!(
                "{}\n\n... [truncated, {} total chars]",
                &stdout[..5000],
                stdout.len()
            ))
        } else {
            Ok(stdout.to_string())
        }
    }
}
