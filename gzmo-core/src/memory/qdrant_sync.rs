//! Push SQLite vault embeddings to Qdrant via the project sync script.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{Context, Result};
use tracing::{info, warn};

use crate::config::QdrantConfig;

/// `None` = sync disabled (no-op). `Some(db)` = resolved vault path ready to spawn.
fn sync_preflight(
    project_root: &Path,
    cfg: &QdrantConfig,
    vault_db: &Path,
) -> Result<Option<PathBuf>> {
    if !cfg.enabled || !cfg.sync_enabled {
        return Ok(None);
    }
    if cfg.url.trim().is_empty() {
        anyhow::bail!("qdrant url is empty");
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
    if !vault_db.is_file() {
        anyhow::bail!("vault db missing: {}", vault_db.display());
    }
    Ok(Some(vault_db))
}

/// Run `scripts/sync-vault-to-qdrant.py` from `project_root`.
pub async fn sync_vault_to_qdrant(
    project_root: &Path,
    cfg: &QdrantConfig,
    vault_db: &Path,
) -> Result<()> {
    sync_vault_to_qdrant_filtered(project_root, cfg, vault_db, None, None).await
}

/// Incremental sync: optional `--since` ISO timestamp and/or `--ids` UUID list.
pub async fn sync_vault_to_qdrant_filtered(
    project_root: &Path,
    cfg: &QdrantConfig,
    vault_db: &Path,
    since: Option<&str>,
    ids: Option<&[String]>,
) -> Result<()> {
    let Some(vault_db) = sync_preflight(project_root, cfg, vault_db)? else {
        return Ok(());
    };
    let script = project_root.join("scripts/sync-vault-to-qdrant.py");

    info!(
        url = %cfg.url,
        collection = %cfg.collection,
        vault = %vault_db.display(),
        since = since.unwrap_or(""),
        ids = ids.map(|i| i.len()).unwrap_or(0),
        "Qdrant vault sync starting"
    );

    let sync_source = if cfg.collection == "honeypot" {
        "honeypot"
    } else {
        "vault"
    };

    let mut cmd = tokio::process::Command::new("python3");
    cmd.arg(&script)
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
        .stderr(Stdio::piped());
    if let Some(s) = since {
        cmd.arg("--since").arg(s);
    }
    if let Some(list) = ids {
        if !list.is_empty() {
            cmd.arg("--ids").arg(list.join(","));
        }
    }

    let output = cmd
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

    // Full sync still gets verify; incremental id/since skips sample verify (cheaper).
    if since.is_none() && ids.is_none() {
        let verify = project_root.join("scripts/qdrant-post-sync-verify.sh");
        if verify.is_file() {
            let verify_out = tokio::process::Command::new("bash")
                .arg(&verify)
                .arg("--gzmo-root")
                .arg(project_root)
                .current_dir(project_root)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
                .with_context(|| format!("spawn {}", verify.display()))?;
            for line in String::from_utf8_lossy(&verify_out.stdout)
                .lines()
                .chain(String::from_utf8_lossy(&verify_out.stderr).lines())
            {
                if !line.is_empty() {
                    info!(line = %line, "qdrant-post-sync-verify");
                }
            }
            if !verify_out.status.success() {
                warn!("Qdrant post-sync sample verify failed");
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::QdrantConfig;
    use std::fs;

    fn cfg(enabled: bool, sync: bool, url: &str) -> QdrantConfig {
        QdrantConfig {
            enabled,
            url: url.into(),
            collection: "vault".into(),
            sync_enabled: sync,
            sync_cron_hour: 0,
            sync_cron_minute: 0,
        }
    }

    #[test]
    fn preflight_disabled_is_noop() {
        let root = std::env::temp_dir();
        assert!(
            sync_preflight(&root, &cfg(false, true, "http://q"), Path::new("x"))
                .unwrap()
                .is_none()
        );
        assert!(
            sync_preflight(&root, &cfg(true, false, "http://q"), Path::new("x"))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn preflight_empty_url_or_missing_db_fails_closed() {
        let root = std::env::temp_dir().join(format!(
            "gzmo-qdrant-sync-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("scripts")).unwrap();
        fs::write(root.join("scripts/sync-vault-to-qdrant.py"), b"#").unwrap();
        let db = root.join("vault.db");

        assert!(sync_preflight(&root, &cfg(true, true, "   "), &db).is_err());
        assert!(sync_preflight(&root, &cfg(true, true, "http://q"), &db).is_err());

        fs::write(&db, b"").unwrap();
        let got = sync_preflight(&root, &cfg(true, true, "http://q"), &db).unwrap();
        assert_eq!(got.as_deref(), Some(db.as_path()));
        let _ = fs::remove_dir_all(&root);
    }
}
