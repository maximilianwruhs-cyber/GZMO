//! Bibliothek promotion gate — tracks dream cycles before stable wiki promotion.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

const STATE_FILE: &str = "data/bibliothek_state.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BibliothekState {
    pub dream_cycles_completed: u32,
}

pub fn default_state_path(project_root: &Path) -> std::path::PathBuf {
    project_root.join(STATE_FILE)
}

pub fn load_state(path: &Path) -> BibliothekState {
    if !path.exists() {
        return BibliothekState::default();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

pub fn save_state(path: &Path, state: &BibliothekState) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}

pub fn increment_dream_cycles(path: &Path) -> anyhow::Result<u32> {
    let mut state = load_state(path);
    state.dream_cycles_completed = state.dream_cycles_completed.saturating_add(1);
    save_state(path, &state)?;
    Ok(state.dream_cycles_completed)
}

/// Whether vault/KG promotion is allowed under Bibliothek policy.
pub fn promotion_allowed(path: &Path, min_dream_cycles: u32) -> bool {
    if min_dream_cycles == 0 {
        return true;
    }
    load_state(path).dream_cycles_completed >= min_dream_cycles
}

/// Würfel-sandbox honeypot tag — generative cron output must not auto-promote.
pub const WUERFEL_SANDBOX_TAG: &str = "wuerfel-sandbox";

/// Synapse source tag for autopoietic dice-loop rolls.
pub const WUERFEL_CRON_SOURCE: &str = "wuerfel-cron";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promotion_gate_blocks_until_threshold() {
        let dir = std::env::temp_dir().join(format!("bib_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bibliothek_state.json");
        assert!(!promotion_allowed(&path, 50));
        for _ in 0..50 {
            increment_dream_cycles(&path).unwrap();
        }
        assert!(promotion_allowed(&path, 50));
        let _ = fs::remove_dir_all(dir);
    }
}
