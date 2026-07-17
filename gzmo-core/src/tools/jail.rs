//! Workspace path jail for filesystem tools.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Result};

/// Restricts tool paths to configured workspace roots.
#[derive(Debug, Clone)]
pub struct PathJail {
    roots: Vec<PathBuf>,
}

impl PathJail {
    /// Build a jail from configured roots (empty → process cwd).
    pub fn from_roots(roots: &[PathBuf]) -> Result<Arc<Self>> {
        let mut resolved = Vec::new();
        if roots.is_empty() {
            let cwd = std::env::current_dir()?;
            resolved.push(cwd.canonicalize().unwrap_or(cwd));
        } else {
            for r in roots {
                let canon = if r.exists() {
                    r.canonicalize().unwrap_or_else(|_| r.clone())
                } else {
                    r.clone()
                };
                resolved.push(canon);
            }
        }
        Ok(Arc::new(Self { roots: resolved }))
    }

    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    /// Resolve `path` and ensure it stays inside a root.
    pub fn check(&self, path: &str) -> Result<PathBuf> {
        let p = Path::new(path);
        let absolute = if p.is_absolute() {
            p.to_path_buf()
        } else {
            std::env::current_dir()?.join(p)
        };
        let normalized = normalize_path(&absolute);

        if absolute.exists() {
            let canon = absolute
                .canonicalize()
                .unwrap_or_else(|_| normalized.clone());
            if !self.is_under_any(&canon) {
                bail!(
                    "Path '{}' escapes workspace jail (roots: {})",
                    path,
                    self.roots_display()
                );
            }
            return Ok(canon);
        }

        // New path: parent (or nearest existing ancestor) must be inside a root.
        let mut walk = absolute.parent().unwrap_or(Path::new("/")).to_path_buf();
        while !walk.exists() {
            if !walk.pop() {
                break;
            }
        }
        let parent_ok = if walk.exists() {
            let parent_canon = walk.canonicalize().unwrap_or(walk);
            self.is_under_any(&parent_canon)
        } else {
            self.is_under_any(&normalized)
        };

        if !parent_ok && !self.is_under_any(&normalized) {
            bail!(
                "Path '{}' escapes workspace jail (roots: {})",
                path,
                self.roots_display()
            );
        }
        Ok(normalized)
    }

    fn is_under_any(&self, path: &Path) -> bool {
        self.roots.iter().any(|r| path.starts_with(r))
    }

    fn roots_display(&self) -> String {
        self.roots
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jail_allows_cwd_relative() {
        let jail = PathJail::from_roots(&[]).unwrap();
        let p = jail.check("Cargo.toml").unwrap();
        assert!(p.ends_with("Cargo.toml") || p.file_name().is_some());
    }

    #[test]
    fn jail_blocks_escape() {
        let jail = PathJail::from_roots(&[]).unwrap();
        let err = jail.check("/etc/passwd").unwrap_err();
        assert!(err.to_string().contains("jail"));
    }
}
