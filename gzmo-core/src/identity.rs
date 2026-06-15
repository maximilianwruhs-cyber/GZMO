//! # OpenClaw Identity
//!
//! SOUL.md hot-reloading, Tabula Rasa initialization, and the Gardening
//! (Nemawashi) directive. This crate owns the immutable identity layer.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::types::SoulContext;
use anyhow::{Context, Result};
use chrono::Utc;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::RwLock;
use tracing::{info, warn};

/// The identity engine: loads SOUL.md and watches it for hot-reload.
pub struct IdentityEngine {
    _soul_path: PathBuf,
    pub soul: Arc<RwLock<SoulContext>>,
    _watcher: Option<RecommendedWatcher>,
}

impl IdentityEngine {
    /// Boot from a Tabula Rasa state: load SOUL.md and prepare the identity.
    pub async fn boot(soul_path: impl AsRef<Path>) -> Result<Self> {
        let soul_path = soul_path.as_ref().to_path_buf();
        let context = load_soul_from_disk(&soul_path).await?;
        info!(
            persona = %context.persona_name,
            directives = context.core_directives.len(),
            "Tabula Rasa boot complete — identity loaded"
        );

        let soul = Arc::new(RwLock::new(context));

        // Set up filesystem watcher for live personality edits
        let watcher = setup_hot_reload(soul_path.clone(), Arc::clone(&soul))?;

        Ok(Self {
            _soul_path: soul_path,
            soul,
            _watcher: Some(watcher),
        })
    }

    /// Get a read-only snapshot of the current soul context.
    pub async fn snapshot(&self) -> SoulContext {
        self.soul.read().await.clone()
    }

    /// Generate the system prompt from the current SOUL context.
    pub async fn system_prompt(&self) -> String {
        let soul = self.soul.read().await;
        format!(
            "You are {}.\n\n## Core Directives\n{}\n\n## Ethical Guardrails\n{}\n\n---\n{}",
            soul.persona_name,
            soul.core_directives
                .iter()
                .map(|d| format!("- {d}"))
                .collect::<Vec<_>>()
                .join("\n"),
            soul.ethical_guardrails
                .iter()
                .map(|g| format!("- {g}"))
                .collect::<Vec<_>>()
                .join("\n"),
            soul.raw_markdown,
        )
    }
}

/// Parse SOUL.md from disk. Expects optional YAML frontmatter delimited
/// by `---`, followed by markdown body.
async fn load_soul_from_disk(path: &Path) -> Result<SoulContext> {
    let raw = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read SOUL.md at {:?}", path))?;

    // Split YAML frontmatter from markdown body
    let (frontmatter, body) = split_frontmatter(&raw);

    let mut persona_name = "GZMO".to_string();
    let mut core_directives: Vec<String> = Vec::new();
    let mut ethical_guardrails: Vec<String> = Vec::new();

    if let Some(fm) = frontmatter {
        match serde_yaml::from_str::<serde_json::Value>(&fm) {
            Ok(yaml) => {
                if let Some(name) = yaml.get("persona").and_then(|v| v.as_str()) {
                    persona_name = name.to_string();
                }
                if let Some(dirs) = yaml.get("directives").and_then(|v| v.as_array()) {
                    core_directives = dirs
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                }
                if let Some(guards) = yaml.get("guardrails").and_then(|v| v.as_array()) {
                    ethical_guardrails = guards
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                }
            }
            Err(e) => {
                anyhow::bail!(
                    "SOUL.md contains malformed YAML frontmatter: {}. \
                     Refusing to boot with a corrupted identity file.",
                    e
                );
            }
        }
    }

    Ok(SoulContext {
        persona_name,
        core_directives,
        ethical_guardrails,
        raw_markdown: body.to_string(),
        loaded_at: Utc::now(),
    })
}

/// Split `---` delimited YAML frontmatter from the markdown body.
fn split_frontmatter(content: &str) -> (Option<String>, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (None, content);
    }

    // Find the closing ---
    if let Some(end) = trimmed[3..].find("---") {
        let fm = trimmed[3..3 + end].trim().to_string();
        let body = &trimmed[3 + end + 3..];
        (Some(fm), body.trim_start())
    } else {
        (None, content)
    }
}

/// Watch SOUL.md for changes and atomically swap the identity on modification.
fn setup_hot_reload(
    soul_path: PathBuf,
    soul: Arc<RwLock<SoulContext>>,
) -> Result<RecommendedWatcher> {
    let path_clone = soul_path.clone();

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(event) = res {
            if matches!(event.kind, EventKind::Modify(_)) {
                let path = path_clone.clone();
                let soul = Arc::clone(&soul);
                tokio::spawn(async move {
                    match load_soul_from_disk(&path).await {
                        Ok(new_ctx) => {
                            info!(
                                persona = %new_ctx.persona_name,
                                "SOUL.md modified — hot-reloading identity"
                            );
                            *soul.write().await = new_ctx;
                        }
                        Err(e) => {
                            warn!("Failed to hot-reload SOUL.md: {e}");
                        }
                    }
                });
            }
        }
    })?;

    watcher.watch(&soul_path, RecursiveMode::NonRecursive)?;
    Ok(watcher)
}
