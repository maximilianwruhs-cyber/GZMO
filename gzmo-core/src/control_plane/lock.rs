//! Exclusive write lock for one vault. Separate file from `vault.db` so SQLite WAL
//! locks are not mixed with the owner flock.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

/// `{vault_db}.write.lock` — presence is not the lock; the flock is.
pub fn vault_write_lock_path(vault_db: &Path) -> PathBuf {
    let mut s = vault_db.as_os_str().to_os_string();
    s.push(".write.lock");
    PathBuf::from(s)
}

/// Held exclusive flock. Released when dropped (process exit or unwind).
pub struct VaultWriteLock {
    _file: File,
    pub path: PathBuf,
}

impl VaultWriteLock {
    /// Fail closed if another `gzmo serve` / `gzmo daemon` already owns this vault.
    pub fn try_acquire(vault_db: &Path) -> Result<Self> {
        let path = vault_write_lock_path(vault_db);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create lock dir {}", parent.display()))?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&path)
            .with_context(|| format!("open vault write lock {}", path.display()))?;
        if let Err(e) = file.try_lock() {
            bail!(
                "vault write lock held — another gzmo serve/daemon owns {} ({e})",
                vault_db.display()
            );
        }
        file.set_len(0)
            .with_context(|| format!("truncate lock {}", path.display()))?;
        writeln!(file, "{}", std::process::id())
            .with_context(|| format!("write pid into {}", path.display()))?;
        file.flush().ok();
        Ok(Self { _file: file, path })
    }
}
