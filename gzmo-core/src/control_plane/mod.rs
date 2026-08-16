//! Owner control plane: exclusive vault flock + Unix-socket memory API.
//!
//! `gzmo serve` / `gzmo daemon` claim the lock and listen. CLI and MCP attach
//! when the socket is live; otherwise they open `PlatformMemory` in-process
//! (lite / telescope fallback). Living hard-fail without a socket is a later graft.

mod client;
mod lock;
mod protocol;
mod server;

pub use client::{clients_enabled, ControlPlaneClient};
pub use lock::{vault_write_lock_path, VaultWriteLock};
pub use protocol::{ControlRequest, ControlResponse, PingBody, VIA_IN_PROCESS, VIA_OWNER};
pub use server::{bind_socket, spawn_server};

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use tracing::info;

use crate::config::GzmoConfig;
use crate::platform_memory::PlatformMemory;

/// Held by the overnight owner for the process lifetime.
pub struct OwnerClaim {
    pub lock: VaultWriteLock,
    pub socket_path: PathBuf,
    server: tokio::task::JoinHandle<Result<()>>,
}

impl Drop for OwnerClaim {
    fn drop(&mut self) {
        self.server.abort();
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

pub fn resolved_socket(config: &GzmoConfig) -> PathBuf {
    config
        .control_plane
        .resolved_socket(&config.memory.vault_db)
}

/// Exclusive flock + listen. Second serve/daemon on the same vault dies here.
pub async fn claim_owner(config: &GzmoConfig) -> Result<OwnerClaim> {
    let lock = VaultWriteLock::try_acquire(&config.memory.vault_db)?;
    let socket_path = resolved_socket(config);
    let platform = Arc::new(PlatformMemory::open_as_owner(config, None).await?);
    let server = spawn_server(socket_path.clone(), platform).await?;
    info!(
        socket = %socket_path.display(),
        lock = %lock.path.display(),
        "vault owner claimed"
    );
    Ok(OwnerClaim {
        lock,
        socket_path,
        server,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GzmoConfig;
    use crate::memory::vault::SqliteVault;
    use std::time::Duration;

    fn lab_config(dir: &std::path::Path) -> GzmoConfig {
        let mut cfg = GzmoConfig::default();
        cfg.memory.vault_db = dir.join("vault.db");
        cfg.memory.directory = dir.join("memory");
        cfg.redis.enabled = false;
        cfg.embeddings.enabled = false;
        cfg.qdrant.enabled = false;
        cfg.rerank.enabled = false;
        cfg.control_plane.socket_path = dir.join("gzmo.sock");
        cfg
    }

    #[test]
    fn second_exclusive_lock_fails() {
        let dir = tempfile_dir();
        let vault = dir.join("vault.db");
        let first = VaultWriteLock::try_acquire(&vault).expect("first lock");
        let second = VaultWriteLock::try_acquire(&vault);
        assert!(second.is_err(), "second owner must fail closed");
        drop(first);
        VaultWriteLock::try_acquire(&vault).expect("lock reusable after drop");
    }

    #[tokio::test]
    async fn socket_ping_status_search() {
        let dir = tempfile_dir();
        let cfg = lab_config(&dir);
        std::fs::create_dir_all(&cfg.memory.directory).unwrap();
        let vault = SqliteVault::open(&cfg.memory.vault_db).unwrap();
        vault
            .store_text(
                "honeypot recall fixture about the owner socket",
                "Semantic",
                0.9,
            )
            .unwrap();
        drop(vault);

        let claim = claim_owner(&cfg).await.expect("claim owner");
        // Accept loop needs a tick to bind before the first client.
        tokio::time::sleep(Duration::from_millis(30)).await;

        let client = ControlPlaneClient::connect_if_live(&cfg, Some("test-session".into()))
            .await
            .expect("live socket");
        let ping = client.ping().await.expect("ping");
        assert_eq!(ping.vault_path, cfg.memory.vault_db.display().to_string());
        assert_eq!(std::path::Path::new(&ping.socket_path), claim.socket_path);

        let st = client.status().await.expect("status");
        assert_eq!(st.control_plane.as_deref(), Some(VIA_OWNER));
        assert!(st.vault_facts >= 1);

        let search = client
            .search("owner socket", 5, false)
            .await
            .expect("search");
        assert!(
            search.hits >= 1,
            "expected a fixture hit, got: {}",
            search.text
        );

        drop(claim);
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(
            ControlPlaneClient::connect_if_live(&cfg, None)
                .await
                .is_none(),
            "socket must die with the owner"
        );
    }

    #[tokio::test]
    async fn connect_if_live_none_without_server() {
        let dir = tempfile_dir();
        let cfg = lab_config(&dir);
        assert!(ControlPlaneClient::connect_if_live(&cfg, None)
            .await
            .is_none());
    }

    fn tempfile_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gzmo-cp-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
