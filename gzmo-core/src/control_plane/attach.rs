//! Client attach policy: owner socket, lite in-process, or living hard-fail.

use std::path::Path;

use anyhow::{bail, Result};

use crate::config::GzmoConfig;

use super::client::{clients_enabled, ControlPlaneClient};
use super::resolved_socket;

#[derive(Debug)]
pub enum MemoryAttach {
    Owner(ControlPlaneClient),
    Local,
}

/// Product/lite vaults live under `~/.gzmo`.
pub fn is_lite_vault(vault_db: &Path) -> bool {
    let Ok(home) = std::env::var("HOME") else {
        return false;
    };
    vault_db.starts_with(Path::new(&home).join(".gzmo"))
}

/// The living animal vault. Telescope `data-next/` is not this.
pub fn is_living_vault(vault_db: &Path) -> bool {
    vault_db.starts_with(Path::new("/opt/gzmo"))
}

/// Attach to the owner socket, or open in-process only for lite / explicit escape.
///
/// Living vault + dead socket + no `--offline` / `GZMO_CONTROL_PLANE=0` → error.
/// `--offline` while the owner is up → error (do not take a WAL slot during metabolism).
pub async fn attach_memory(
    config: &GzmoConfig,
    session_id: Option<String>,
    offline: bool,
) -> Result<MemoryAttach> {
    let socket = resolved_socket(config);
    let live = ControlPlaneClient::connect_if_live(config, session_id.clone()).await;

    if offline {
        if live.is_some() {
            bail!(
                "owner is up at {} — refuse --offline while the socket is live",
                socket.display()
            );
        }
        return Ok(MemoryAttach::Local);
    }

    if let Some(client) = live {
        return Ok(MemoryAttach::Owner(client));
    }

    if !clients_enabled()
        || is_lite_vault(&config.memory.vault_db)
        || !is_living_vault(&config.memory.vault_db)
    {
        return Ok(MemoryAttach::Local);
    }

    bail!(
        "living vault {} has no owner at {} — start gzmo daemon, or --offline for inspect, or GZMO_CONTROL_PLANE=0",
        config.memory.vault_db.display(),
        socket.display()
    )
}
