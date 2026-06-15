//! Push SQLite vault embeddings to Qdrant via the project sync script.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{Context, Result};
use tracing::{info, warn};

use crate::config::QdrantConfig;

/// Run `scripts/sync-vault-to-qdrant.py` from `project_root`.
pub async fn sync_vault_to_qdrant(project_root: &Path, cfg: &QdrantConfig, vault_db: &Path) -> Result<()> {
    if !cfg.enabled || !cfg.sync_enabled {
        return Ok(());
    }

    let script = project_root.join("scripts/sync-vault-to-qdrant.py");
    if !script.is_file() {
        anyhow::bail!("Missing sync script: {}", script.display());
    }

    let vault_db = if vault_db.is_absolute() {
        vault_db.to_path_buf()
    } else {
        project_root.join(vault_db)
    };

    info!(
        url = %cfg.url,
        collection = %cfg.collection,
        vault = %vault_db.display(),
        "Qdrant vault sync starting"
    );

    let sync_source = if cfg.collection == "honeypot" {
        "honeypot"
    } else {
        "vault"
    };

    let output = tokio::process::Command::new("python3")
        .arg(&script)
        .arg("--db")
        .arg(&vault_db)
        .arg("--url")
        .arg(&cfg.url)
        .arg("--collection")
        .arg(&cfg.collection)
        .arg("--source")
        .arg(sync_source)
        .current_dir(project_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .with_context(|| format!("spawn {}", script.display()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    for line in stdout.lines().chain(stderr.lines()) {
        if !line.is_empty() {
            info!(line = %line, "qdrant-sync");
        }
    }

    if !output.status.success() {
        warn!(code = ?output.status.code(), "Qdrant vault sync failed");
        anyhow::bail!("sync-vault-to-qdrant exited with {}", output.status);
    }

    Ok(())
}

/// Resolve GZMO project root (directory containing `gzmo.toml`).
pub fn discover_project_root() -> PathBuf {
    if let Ok(dir) = std::env::var("GZMO_PROJECT_ROOT") {
        let p = PathBuf::from(dir);
        if p.join("gzmo.toml").is_file() {
            return p;
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
